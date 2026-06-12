//! The Clipboard Ring controller: hotkey dispatch, scope changes, dismiss, jump.

use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tracing::{info, warn};

use crate::clipboard::watcher;
use crate::db::repo;
use crate::ring::buffer::{ClipId, RingBuffer, RingConfig, RingScope};
use crate::ring::writer;
use crate::state::AppState;

/// A handle to the live ring. Cheap to clone; cheap to lock (parking_lot).
#[derive(Clone)]
pub struct RingController {
    inner: Arc<Mutex<RingBuffer>>,
}

impl RingController {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RingBuffer::new(
                RingScope::default(),
                RingConfig::default(),
            ))),
        }
    }

    pub fn lock(&self) -> parking_lot::MutexGuard<'_, RingBuffer> {
        self.inner.lock()
    }

    /// Set the current scope and rebuild the slot list from the database.
    pub fn set_scope<R: Runtime>(
        &self,
        _app: &AppHandle<R>,
        state: &AppState,
        scope: RingScope,
        config: RingConfig,
    ) {
        let cap = config.capacity;
        let items = match scope {
            RingScope::Global => repo::list_recent(state, &config),
            RingScope::Favorites => repo::list_recent_favorites(state, cap),
            RingScope::Collection(ref cid) => repo::list_recent_by_collection(state, cid, cap),
            RingScope::Application { ref exe } => repo::list_recent_by_source_app(state, exe, cap),
            RingScope::Kind(ref k) => repo::list_recent_by_kind(state, k, cap),
            RingScope::NamedSet(ref id) => repo::list_ring_set_items(state, id, cap),
        };
        let items = items.unwrap_or_else(|e| {
            warn!(?e, "ring: failed to list items; clearing");
            Vec::new()
        });
        let mut guard = self.inner.lock();
        guard.scope = scope;
        guard.config = config;
        guard.rebuild(items);
        info!(count = guard.slot_count(), "ring rebuilt");
    }

    /// Invalidate the cached slots. Called by the watcher when a new clip is recorded.
    pub fn invalidate(&self) {
        let mut guard = self.inner.lock();
        guard.invalidate();
    }

    /// Returns true if the cached slot list is stale and should be rebuilt.
    pub fn is_dirty(&self) -> bool {
        self.inner.lock().is_dirty()
    }

    /// Ensure the slot list is fresh, rebuilding it if marked dirty.
    fn ensure_fresh<R: Runtime>(&self, app: &AppHandle<R>, state: &AppState) {
        let (scope, cfg) = {
            let guard = self.inner.lock();
            if !guard.is_dirty() && guard.slot_count() > 0 {
                return;
            }
            (guard.scope.clone(), guard.config.clone())
        };
        self.set_scope(app, state, scope, cfg);
    }

    /// Cycle to the *older* slot, write it to the OS clipboard.
    pub fn reverse<R: Runtime>(&self, app: &AppHandle<R>, state: &AppState) -> RingActionResult {
        self.ensure_fresh(app, state);
        let id = {
            let mut guard = self.inner.lock();
            guard.reverse().cloned()
        };
        self.activate_slot(app, state, id)
    }

    /// Cycle to the *newer* slot, write it to the OS clipboard.
    pub fn forward<R: Runtime>(&self, app: &AppHandle<R>, state: &AppState) -> RingActionResult {
        self.ensure_fresh(app, state);
        let id = {
            let mut guard = self.inner.lock();
            guard.forward().cloned()
        };
        self.activate_slot(app, state, id)
    }

    /// Jump to a specific slot by index.
    pub fn jump<R: Runtime>(&self, app: &AppHandle<R>, state: &AppState, index: usize) -> RingActionResult {
        let id = {
            let mut guard = self.inner.lock();
            guard.jump(index).cloned()
        };
        self.activate_slot(app, state, id)
    }

    /// Dismiss the ring.
    pub fn dismiss(&self) {
        self.inner.lock().dismiss();
    }

    /// Returns true if the ring is currently active.
    pub fn is_active(&self) -> bool {
        self.inner.lock().is_active()
    }

    /// Returns true if the ring has been idle long enough to be auto-dismissed.
    pub fn is_idle_timed_out(&self) -> bool {
        self.inner.lock().is_idle_timed_out()
    }

    /// Return the next `n` slot previews for the overlay UI.
    pub fn preview(&self, n: usize) -> Vec<crate::ring::buffer::RingSlotView> {
        self.inner.lock().preview(n)
    }

    /// Look up the slot's clip and write it to the OS clipboard. Emits a
    /// `clip://ring-rotated` event on success.
    fn activate_slot<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        state: &AppState,
        slot_id: Option<ClipId>,
    ) -> RingActionResult {
        let Some(id) = slot_id else {
            self.emit_ring_empty(app);
            return RingActionResult::Empty;
        };
        let conn = match state.db.get() {
            Ok(c) => c,
            Err(e) => return RingActionResult::Failed(e.to_string()),
        };
        let clip = match repo::get_clip(&conn, &id) {
            Ok(Some(c)) => c,
            Ok(None) => return RingActionResult::Pruned,
            Err(e) => return RingActionResult::Failed(e.to_string()),
        };
        drop(conn);

        // Skip the write if the OS clipboard already matches the slot.
        if writer::os_clipboard_matches(&clip) {
            let (index, total) = self.cursor_info();
            let _ = app.emit("clip://ring-rotated", serde_json::json!({
                "id": clip.id,
                "index": index,
                "total": total,
                "no_op": true,
            }));
            return RingActionResult::NoOp;
        }

        let outcome = writer::write_clip_to_clipboard(state, &clip);
        match outcome {
            writer::WriteOutcome::Written => {
                let (index, total) = self.cursor_info();
                let _ = app.emit("clip://ring-rotated", serde_json::json!({
                    "id": clip.id,
                    "index": index,
                    "total": total,
                    "no_op": false,
                }));
                writer::log_rotated(&clip.id, index, total);

                // Optional auto-paste: if the user opted in, simulate Ctrl+V
                // so the active window receives the ring slot. Done off the
                // hotkey thread so the IPC callback returns immediately.
                let settings = crate::settings::load_settings(app);
                if settings.auto_paste {
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(60));
                        crate::paste::send_ctrl_v();
                    });
                }

                RingActionResult::Activated { id: clip.id, index, total }
            }
            writer::WriteOutcome::AlreadyCurrent => RingActionResult::NoOp,
            writer::WriteOutcome::Skipped => RingActionResult::Skipped,
            writer::WriteOutcome::Failed(e) => RingActionResult::Failed(e),
        }
    }

    fn cursor_info(&self) -> (usize, usize) {
        let guard = self.inner.lock();
        let total = guard.slot_count();
        let index = guard.active_index().unwrap_or(0);
        (index, total)
    }

    fn emit_ring_empty<R: Runtime>(&self, app: &AppHandle<R>) {
        let _ = app.emit("clip://ring-empty", serde_json::json!({}));
    }
}

impl Default for RingController {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a ring action (cycle forward/back/jump).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RingActionResult {
    Activated {
        id: String,
        index: usize,
        total: usize,
    },
    NoOp,
    /// The ring was active but the slot's clip was pruned from the DB.
    Pruned,
    /// The ring is empty.
    Empty,
    /// The slot exists but its payload was skipped (e.g. no preview).
    Skipped,
    /// An error occurred (db / arboard).
    Failed(String),
}

/// Helper: listen for the existing `clip://created` and `clip://updated` events
/// to invalidate the ring. Called from `lib::run`. The returned
/// `(EventId, EventId)` tuple holds the registration tokens so callers can
/// detach the listeners if they ever need to (currently we don't, but it's
/// good hygiene to return them).
pub fn attach_listeners(app: &AppHandle, ring: RingController) -> (tauri::EventId, tauri::EventId) {
    use tauri::Listener;
    let r1 = ring.clone();
    let id1 = app.listen("clip://created", move |_event| {
        // Catch any panic to avoid bringing the whole event loop down.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            r1.invalidate();
        }));
        if let Err(e) = result {
            tracing::warn!("ring invalidate panicked: {:?}", e);
        }
    });
    let r2 = ring.clone();
    let id2 = app.listen("clip://updated", move |_event| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            r2.invalidate();
        }));
        if let Err(e) = result {
            tracing::warn!("ring invalidate panicked: {:?}", e);
        }
    });
    // Persist a reference so we can find it later if needed.
    app.manage(ring);
    (id1, id2)
}

/// Resolves the controller from a `Manager` (panics if not registered, which
/// only happens if the lib wasn't properly set up).
pub fn get_controller<R: tauri::Runtime>(app: &impl Manager<R>) -> RingController {
    app.state::<RingController>().inner().clone()
}

/// Helper used by the watcher to suppress after a ring write. Currently a
/// re-export of the existing `arm_suppress` to keep callers simple.
pub fn suppress_for_hash(state: &AppState, hash: String) {
    watcher::arm_suppress(state, hash);
}

/// Convenience: returns true if the ring is currently active (cursor is set).
pub fn is_ring_active<R: tauri::Runtime>(app: &AppHandle<R>) -> bool {
    if let Some(ring) = app.try_state::<RingController>() {
        ring.is_active()
    } else {
        false
    }
}

/// Convenience: cycle the ring in either direction. The actual result is
/// emitted as a `clip://ring-rotated` / `clip://ring-empty` event by the
/// controller itself.
pub fn cycle_ring<R: tauri::Runtime>(app: &AppHandle<R>, forward: bool) {
    use tauri::Manager;
    let Some(ring) = app.try_state::<RingController>() else {
        return;
    };
    let Some(state) = app.try_state::<std::sync::Arc<AppState>>() else {
        return;
    };
    if forward {
        ring.forward(app, &**state);
    } else {
        ring.reverse(app, &**state);
    }
}

/// Convenience: get a handle to the registered ring controller.
pub fn get_ring<R: tauri::Runtime>(app: &AppHandle<R>) -> RingController {
    use tauri::Manager;
    app.state::<RingController>().inner().clone()
}

/// Periodic timer that dismisses the ring if it has been idle for
/// `idle_dismiss_ms` milliseconds. Spawned once during `attach` and lives
/// for the lifetime of the app.
pub fn start_idle_timer(app: AppHandle) {
    use std::time::Duration;
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        // Use a fresh state lookup so we always talk to the live controller.
        let ring = match app.try_state::<RingController>() {
            Some(s) => s.inner().clone(),
            None => return, // Controller was removed; the app is shutting down.
        };
        if ring.is_idle_timed_out() {
            ring.dismiss();
            let _ = app.emit("clip://ring-dismissed", serde_json::json!({}));
        }
    });
}
