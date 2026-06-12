//! Clipboard write path used by the Clipboard Ring.
//!
//! The writer is a thin wrapper around `arboard::Clipboard` that knows about
//! the three payload kinds (text/url, image, files) and arms the watcher
//! suppress flag so the ring's own output is not re-recorded.

use arboard::Clipboard;

use crate::clipboard::watcher;
use crate::db::repo;
use crate::state::{AppState, Clip};

/// Result of a write attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    /// Wrote the new payload and armed the suppress flag.
    Written,
    /// The slot's content already matches the OS clipboard; no write happened.
    AlreadyCurrent,
    /// The slot is empty (dismissed) or the content has been pruned.
    Skipped,
    /// An I/O / arboard error happened. The string is the error message.
    Failed(String),
}

/// Write the given clip to the OS clipboard, using the watcher suppress flag so
/// the ring's own output is not re-recorded.
pub fn write_clip_to_clipboard(
    state: &AppState,
    clip: &Clip,
) -> WriteOutcome {
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(e) => return WriteOutcome::Failed(e.to_string()),
    };
    // Always clear first so a partial / wrong-format payload from the previous
    // slot doesn't linger on the clipboard.
    let _ = clipboard.clear();

    let result = match clip.kind.as_str() {
        "text" | "url" => match &clip.text_preview {
            Some(text) if !text.is_empty() => clipboard
                .set_text(text.clone())
                .map_err(|e| e.to_string()),
            _ => return WriteOutcome::Skipped,
        },
        "image" => write_image(state, &mut clipboard, clip).map_err(|e| e.to_string()),
        "files" => write_files(&mut clipboard, clip).map_err(|e| e.to_string()),
        other => return WriteOutcome::Failed(format!("unsupported kind '{other}'")),
    };

    if let Err(e) = result {
        return WriteOutcome::Failed(e);
    }

    // Arm the suppress flag so the watcher's next poll doesn't re-record this
    // very content.
    watcher::arm_suppress(state, clip.content_hash.clone());
    // Bump usage so the cycling history surfaces in `usage_count`.
    if let Ok(conn) = state.db.get() {
        let _ = repo::bump_usage(&conn, &clip.id, repo::now_ms());
    }
    WriteOutcome::Written
}

fn write_image(
    state: &AppState,
    clipboard: &mut Clipboard,
    clip: &Clip,
) -> anyhow::Result<()> {
    let meta = clip
        .image
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("image clip {} has no metadata", clip.id))?;
    let path = state.data_dir.join("images").join(&meta.path);
    let bytes = std::fs::read(&path)
        .map_err(|e| anyhow::anyhow!("read image {}: {e}", path.display()))?;
    let img = image::load_from_memory(&bytes)?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let copied = arboard::ImageData {
        width: w,
        height: h,
        bytes: rgba.into_raw().into(),
    };
    clipboard.set_image(copied)?;
    Ok(())
}

fn write_files(_clipboard: &mut Clipboard, clip: &Clip) -> anyhow::Result<()> {
    let paths = clip.file_paths.as_deref().unwrap_or(&[]);
    if paths.is_empty() {
        return Err(anyhow::anyhow!("file clip has no paths"));
    }
    #[cfg(windows)]
    {
        crate::clipboard::files::write_file_list(paths)?;
    }
    #[cfg(not(windows))]
    {
        // On non-Windows we fall back to writing the joined path list as text.
        let joined = paths.join("\n");
        _clipboard.set_text(joined)?;
    }
    Ok(())
}

/// Best-effort check: read the current OS clipboard's text and compare it
/// against the slot's preview. Returns `true` if they look the same, in which
/// case the ring should skip the write.
pub fn os_clipboard_matches(clip: &Clip) -> bool {
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(_) => return false,
    };
    match clip.kind.as_str() {
        "text" | "url" => {
            let current = clipboard.get_text().unwrap_or_default();
            // The OS may add a trailing NUL; trim it.
            let trimmed = current.trim_end_matches('\u{0}');
            let preview = clip.text_preview.as_deref().unwrap_or("");
            if preview.len() <= 200 {
                trimmed == preview
            } else {
                // `trimmed` is a `&str` so this slicing respects UTF-8 boundaries.
                let preview_prefix = preview.get(..200).unwrap_or(preview);
                trimmed == preview_prefix
            }
        }
        "image" => clipboard.get_image().is_ok(),
        "files" => false,
        _ => false,
    }
}

// Note: a small per-process image cache could be added here if profiling shows
// the image-decode path is hot. For now the decode is gated by the watcher's
// suppress flag, so a single ring rotation does at most one decode per slot.

/// Emit a small "ring rotated" log line. Useful for support diagnostics.
pub fn log_rotated(id: &str, index: usize, total: usize) {
    tracing::info!(id, index, total, "ring rotated");
}
