//! Background polling thread that detects clipboard changes and records them.

use arboard::Clipboard;
use tauri::{AppHandle, Emitter};
use tracing::{debug, info, warn};

use crate::clipboard::source;
use crate::clipboard::{DEDUPE_WINDOW_MS, POLL_INTERVAL};
use crate::db::repo;
use crate::state::AppState;

/// Spawn the watcher thread. Idempotent: if one is already running, this is a no-op.
pub fn start(app: AppHandle, state: AppState) {
    if state.is_watcher_running() {
        return;
    }
    state.set_watcher_running(true);
    let app_handle = app.clone();
    let state_clone = state.clone();
    std::thread::Builder::new()
        .name("clipvault-clipboard-watcher".into())
        .spawn(move || {
            if let Err(e) = run_loop(app_handle.clone(), state_clone.clone()) {
                warn!(?e, "clipboard watcher exited with error");
            }
            state_clone.set_watcher_running(false);
            info!("clipboard watcher thread terminated");
        })
        .expect("failed to spawn clipboard watcher thread");

    // Also emit a startup ping so the frontend knows we're alive.
    let _ = app.emit("clip://ready", serde_json::json!({ "ok": true }));
}

fn run_loop(app: AppHandle, state: AppState) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    let mut last_text_hash: Option<String> = None;
    let mut last_image_hash: Option<String> = None;
    let mut last_files_hash: Option<String> = None;

    loop {
        std::thread::sleep(POLL_INTERVAL);

        // Capture the foreground app once per poll so all three branches share the same source.
        let (source_app, source_title) = source::current_source();

        // 1. Text
        match clipboard.get_text() {
            Ok(text) => {
                let trimmed = text.trim_end_matches('\u{0}').to_string();
                if !trimmed.is_empty() {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    if last_text_hash.as_deref() != Some(&hash) {
                        last_text_hash = Some(hash.clone());
                        if !state.suppress.should_skip(&hash) {
                            if let Err(e) = record_text(&app, &state, &trimmed, &hash, source_app.as_deref(), source_title.as_deref()) {
                                warn!(?e, "failed to record text clip");
                            }
                        }
                    }
                }
            }
            Err(e) => debug!(?e, "no text on clipboard"),
        }

        // 2. Image
        match clipboard.get_image() {
            Ok(img) => {
                let hash = blake3::hash(&img.bytes).to_hex().to_string();
                if last_image_hash.as_deref() != Some(&hash) {
                    last_image_hash = Some(hash.clone());
                    if !state.suppress.should_skip(&hash) {
                        if let Err(e) = record_image(&app, &state, &img.bytes, img.width as u32, img.height as u32, &hash, source_app.as_deref(), source_title.as_deref()) {
                            warn!(?e, "failed to record image clip");
                        }
                    }
                }
            }
            Err(_) => { /* no image */ }
        }

        // 3. File copies (Windows-specific, optional on other platforms)
        #[cfg(windows)]
        {
            if let Some(paths) = crate::clipboard::files::read_file_list() {
                if !paths.is_empty() {
                    let joined = paths.join("|");
                    let hash = blake3::hash(joined.as_bytes()).to_hex().to_string();
                    if last_files_hash.as_deref() != Some(&hash) {
                        last_files_hash = Some(hash.clone());
                        if !state.suppress.should_skip(&hash) {
                            if let Err(e) = record_files(&app, &state, &paths, &hash, source_app.as_deref(), source_title.as_deref()) {
                                warn!(?e, "failed to record file clip");
                            }
                        }
                    }
                }
            }
        }

        // Keep the app from being killed while the watcher is running.
        if state_clone_watcher_should_stop(&app) {
            return Ok(());
        }
    }
}

fn state_clone_watcher_should_stop(_app: &AppHandle) -> bool {
    // Hook for future graceful shutdown. Returns false for now (watcher runs forever).
    false
}

fn record_text(
    app: &AppHandle,
    state: &AppState,
    text: &str,
    hash: &str,
    source_app: Option<&str>,
    source_title: Option<&str>,
) -> anyhow::Result<()> {
    let kind = if url::Url::parse(text.trim()).is_ok() && !text.contains('\n') {
        "url"
    } else {
        "text"
    };
    let preview: String = text.chars().take(200).collect();
    let now = repo::now_ms();
    let conn = state.db.get()?;
    if let Some(existing) = repo::find_recent_duplicate(&conn, hash, DEDUPE_WINDOW_MS, now)? {
        repo::bump_usage(&conn, &existing, now)?;
        let _ = app.emit("clip://updated", serde_json::json!({ "id": existing }));
        return Ok(());
    }
    let id = repo::insert_clip(
        &conn,
        kind,
        hash,
        Some(&preview),
        text.len() as i64,
        source_app,
        source_title,
        now,
    )?;
    if let Some(clip) = repo::get_clip(&conn, &id)? {
        let _ = app.emit("clip://created", clip);
    }
    Ok(())
}

fn record_image(
    app: &AppHandle,
    state: &AppState,
    rgba_bytes: &[u8],
    width: u32,
    height: u32,
    hash: &str,
    source_app: Option<&str>,
    source_title: Option<&str>,
) -> anyhow::Result<()> {
    let now = repo::now_ms();
    let conn = state.db.get()?;
    if let Some(existing) = repo::find_recent_duplicate(&conn, hash, DEDUPE_WINDOW_MS, now)? {
        repo::bump_usage(&conn, &existing, now)?;
        let _ = app.emit("clip://updated", serde_json::json!({ "id": existing }));
        return Ok(());
    }
    // The clipboard gave us raw RGBA pixels, not an encoded file. We re-encode
    // to PNG on disk so the saved file is a real, portable image that any
    // tool can re-open. Byte size is reported as the raw RGBA size for
    // consistency with how `arboard` measured it.
    let id = repo::insert_clip(
        &conn,
        "image",
        hash,
        None,
        rgba_bytes.len() as i64,
        source_app,
        source_title,
        now,
    )?;
    let images_dir = state.data_dir.join("images");
    std::fs::create_dir_all(&images_dir)?;
    let (path, thumb_path) =
        crate::images::storage::save_image(&images_dir, &id, width, height, rgba_bytes)?;
    repo::attach_image(
        &conn,
        &id,
        &path,
        &thumb_path,
        width as i64,
        height as i64,
        "image/png",
    )?;
    if let Some(clip) = repo::get_clip(&conn, &id)? {
        let _ = app.emit("clip://created", clip);
    }
    Ok(())
}

fn record_files(
    app: &AppHandle,
    state: &AppState,
    paths: &[String],
    hash: &str,
    source_app: Option<&str>,
    source_title: Option<&str>,
) -> anyhow::Result<()> {
    let now = repo::now_ms();
    let conn = state.db.get()?;
    if let Some(existing) = repo::find_recent_duplicate(&conn, hash, DEDUPE_WINDOW_MS, now)? {
        repo::bump_usage(&conn, &existing, now)?;
        let _ = app.emit("clip://updated", serde_json::json!({ "id": existing }));
        return Ok(());
    }
    let json = serde_json::to_string(paths)?;
    let total_size: i64 = paths
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len() as i64)
        .sum();
    let preview = paths
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let id = repo::insert_clip(
        &conn,
        "files",
        hash,
        Some(&preview),
        total_size,
        source_app,
        source_title,
        now,
    )?;
    repo::attach_files(&conn, &id, &json)?;
    if let Some(clip) = repo::get_clip(&conn, &id)? {
        let _ = app.emit("clip://created", clip);
    }
    Ok(())
}

/// Re-arm the suppress flag for the given content. Used by the Quick Paste flow so the
/// watcher doesn't immediately re-record the just-pasted content.
pub fn arm_suppress(state: &AppState, hash: String) {
    state.suppress.arm(hash, crate::clipboard::SUPPRESS_TTL_MS);
}
