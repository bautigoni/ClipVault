//! Clipboard Ring: a power-user way to cycle through recent clips via hotkey.
//!
//! ```text
//!   global-shortcut plugin
//!           │
//!           ▼
//!   controller::RingController  ◀─── AppState.ring
//!           │
//!           ├── ring::buffer    (in-memory slot list, cursor, scope)
//!           ├── ring::writer    (clipboard write + suppress arm)
//!           └── ring::overlay   (UI event fan-out)
//! ```
//!
//! The ring is a *view* over the persistent `clips` table; the slot list is
//! rebuilt on demand (cold start, scope change, watcher invalidation) by
//! calling [`controller::RingController::set_scope`].

pub mod buffer;
pub mod controller;
pub mod overlay;
pub mod writer;

pub use buffer::{
    ClipId, ClipType, CollectionId, RingBuffer, RingConfig, RingScope, RingSetId, RingSlotView,
    SlotData,
};
pub use controller::{RingActionResult, RingController};

use std::sync::Arc;

use tauri::AppHandle;

use crate::state::AppState;

/// Initialise the ring subsystem. Returns a [`RingController`] that should be
/// stored on [`AppState`](crate::state::AppState) and listened to for
/// `clip://created` / `clip://updated` events.
pub fn init() -> RingController {
    RingController::new()
}

/// Default scope and config used on first launch.
pub fn default_scope() -> (RingScope, RingConfig) {
    (RingScope::Global, RingConfig::default())
}

/// Convenience wrapper for `repo::list_recent` so callers can swap to in-memory
/// filtering later. Currently a no-op (the controller decides when to rebuild).
pub fn refresh_if_needed(_state: &AppState, _ring: &RingController) {}

/// Helper to attach event listeners. Called from `lib::run` after the
/// controller is stored on AppState. Returns the (created, updated) `EventId`
/// tokens so callers can unregister the listeners later if they need to.
pub fn attach(
    app: &AppHandle,
    ring: RingController,
) -> (tauri::EventId, tauri::EventId) {
    controller::attach_listeners(app, ring)
}

/// Placeholder: the `Arc<...>` type stored in `AppState`.
pub type SharedRing = Arc<RingController>;
