//! The optional ring overlay: a slim top-of-screen bar that shows the next few
//! ring slots. We piggy-back on the existing `palette` webview — no new
//! Tauri window — by emitting `clip://ring-rotated` events that the React
//! frontend already subscribes to (see `useClipsInvalidation`).
//!
//! This module is a placeholder for the actual UI work in `palette.tsx` and
//! keeps the Rust side ready by re-emitting the right event types.

use tauri::{AppHandle, Emitter, Runtime};
use tracing::info;

use crate::ring::buffer::RingSlotView;
use crate::ring::controller::RingController;

/// Open the overlay preview. For v1 we just emit a `clip://ring-preview` event
/// that the palette can listen to. The Rust side does not need to spawn a
/// new window — the palette is always alive (hidden) and ready to render.
pub fn show_preview<R: Runtime>(app: &AppHandle<R>, ring: &RingController) -> Vec<RingSlotView> {
    let slots = ring.preview(5);
    let _ = app.emit("clip://ring-preview", &slots);
    info!(count = slots.len(), "ring preview requested");
    slots
}

/// Dismiss the ring and emit a single event so the frontend can hide any UI.
pub fn dismiss<R: Runtime>(app: &AppHandle<R>, ring: &RingController) {
    ring.dismiss();
    let _ = app.emit("clip://ring-dismissed", serde_json::json!({}));
}
