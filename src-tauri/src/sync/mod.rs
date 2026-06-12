//! Optional self-hosted sync (feature `sync`). The protocol is intentionally minimal:
//! the client POSTs new/changed clips to the configured endpoint, and the server
//! reconciles by `(client_id, clip_id)` pairs. Disabled by default.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnvelope {
    pub client_id: String,
    pub pushed_at: i64,
    pub clips: Vec<serde_json::Value>,
}

pub fn push_envelope(_endpoint: &str, _envelope: &SyncEnvelope) -> anyhow::Result<()> {
    // Implementation: use `tauri_plugin_http` to POST `_envelope` to `_endpoint`.
    // Out of scope for the first build; feature-gated.
    Ok(())
}
