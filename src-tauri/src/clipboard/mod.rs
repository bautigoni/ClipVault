//! Clipboard capture. The watcher polls the system clipboard on a background thread
//! and records new content into the database, deduping by content hash and honoring a
//! suppress flag (so Quick Paste actions don't get re-recorded).

pub mod files;
pub mod source;
pub mod watcher;

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const POLL_INTERVAL: Duration = Duration::from_millis(500);
pub const DEDUPE_WINDOW_MS: i64 = 60_000;
pub const SUPPRESS_TTL_MS: u64 = 1_500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapturedContent {
    Text { text: String, is_url: bool },
    Image { png: Vec<u8>, width: u32, height: u32 },
    Files { paths: Vec<String> },
    Empty,
}

impl CapturedContent {
    pub fn is_empty(&self) -> bool {
        matches!(self, CapturedContent::Empty)
    }
}
