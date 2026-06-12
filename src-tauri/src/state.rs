//! Shared application state, kept inside an `Arc` and accessed by all Tauri commands.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::DbPool;

/// Identifier of the previous clipboard content, used to suppress duplicate inserts
/// from the watcher thread.
pub struct SuppressFlag {
    inner: Mutex<Option<SuppressState>>,
}

struct SuppressState {
    hash: String,
    expires_at: Instant,
}

impl SuppressFlag {
    pub fn new() -> Self {
        Self { inner: Mutex::new(None) }
    }

    /// Returns true if the watcher should skip recording the given hash because
    /// we just wrote it ourselves (Quick Paste flow).
    pub fn should_skip(&self, hash: &str) -> bool {
        let mut guard = self.inner.lock();
        if let Some(state) = guard.as_ref() {
            if state.expires_at > Instant::now() && state.hash == hash {
                *guard = None;
                return true;
            }
        }
        false
    }

    pub fn arm(&self, hash: String, ttl_ms: u64) {
        *self.inner.lock() = Some(SuppressState {
            hash,
            expires_at: Instant::now() + std::time::Duration::from_millis(ttl_ms),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipType {
    Text,
    Image,
    Files,
    Url,
}

impl ClipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipType::Text => "text",
            ClipType::Image => "image",
            ClipType::Files => "files",
            ClipType::Url => "url",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(ClipType::Text),
            "image" => Some(ClipType::Image),
            "files" => Some(ClipType::Files),
            "url" => Some(ClipType::Url),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub content_hash: String,
    pub text_preview: Option<String>,
    pub byte_size: i64,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub is_favorite: bool,
    pub is_pinned: bool,
    pub is_sensitive: bool,
    pub collection_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub usage_count: i64,
    pub last_used_at: Option<i64>,
    pub pinned_at: Option<i64>,
    pub tags: Vec<String>,
    pub image: Option<ImageMeta>,
    pub file_paths: Option<Vec<String>>,
    pub collection_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMeta {
    pub path: String,
    pub thumb_path: String,
    pub width: i64,
    pub height: i64,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: i64,
    pub clip_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub language: String,
    pub body: String,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchPage {
    pub items: Vec<Clip>,
    pub total: i64,
    pub next_cursor: Option<String>,
    pub took_ms: u64,
}

/// The bag of state shared across threads.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub data_dir: PathBuf,
    pub suppress: std::sync::Arc<SuppressFlag>,
    pub watcher_running: std::sync::Arc<AtomicBool>,
    pub app: AppHandle,
}

impl AppState {
    pub fn new(db: DbPool, data_dir: PathBuf, app: AppHandle) -> Self {
        Self {
            db,
            data_dir,
            suppress: std::sync::Arc::new(SuppressFlag::new()),
            watcher_running: std::sync::Arc::new(AtomicBool::new(false)),
            app,
        }
    }

    pub fn is_watcher_running(&self) -> bool {
        self.watcher_running.load(Ordering::SeqCst)
    }

    pub fn set_watcher_running(&self, value: bool) {
        self.watcher_running.store(value, Ordering::SeqCst);
    }
}
