//! Optional local HTTP receiver (feature `http_receiver`) for the browser extension.
//! Binds to 127.0.0.1 on a configurable port and accepts POSTs with clip payloads.

use tauri::AppHandle;

pub fn start(_app: AppHandle, _state: std::sync::Arc<crate::state::AppState>) -> anyhow::Result<()> {
    // Implementation: spin up a tiny hyper / axum server bound to 127.0.0.1.
    // Out of scope for the first build; feature-gated.
    Ok(())
}
