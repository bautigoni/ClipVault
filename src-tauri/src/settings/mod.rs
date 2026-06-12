//! Application settings + retention sweeper + backup scheduler.

pub mod backup;
pub mod retention;
pub mod types;

use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

pub use types::Settings;

const STORE_FILE: &str = "settings.json";

/// Resolve the data directory (where the SQLite DB, images, settings, etc. live).
/// In portable builds (feature `portable`), the data lives next to the executable
/// instead of in `%APPDATA%`.
pub fn data_dir(_app: &AppHandle) -> anyhow::Result<PathBuf> {
    #[cfg(feature = "portable")]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return Ok(parent.join("data"));
            }
        }
    }
    let dir = dirs::data_dir()
        .or_else(dirs::config_dir)
        .context("could not resolve user data directory")?
        .join("com.clipvault.app");
    Ok(dir)
}

pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Settings {
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };
    match store.get("settings") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => Settings::default(),
    }
}

pub fn save_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> anyhow::Result<()> {
    let store = app.store(STORE_FILE)?;
    store.set("settings", serde_json::to_value(settings)?);
    store.save().context("failed to persist settings")?;
    Ok(())
}

pub fn merge_settings(current: &Settings, patch: &SettingsPatch) -> Settings {
    let mut next = current.clone();
    if let Some(v) = patch.retention_days {
        next.retention_days = v;
    }
    if let Some(v) = patch.max_clips {
        next.max_clips = v;
    }
    if let Some(v) = &patch.hotkey {
        next.hotkey = v.clone();
    }
    if let Some(v) = patch.theme {
        next.theme = v;
    }
    if let Some(v) = patch.autostart {
        next.autostart = v;
    }
    if let Some(v) = &patch.storage_dir {
        next.storage_dir = Some(v.clone());
    }
    if let Some(v) = &patch.excluded_apps {
        next.excluded_apps = v.clone();
    }
    if let Some(v) = &patch.sensitive_apps {
        next.sensitive_apps = v.clone();
    }
    if let Some(v) = patch.auto_paste {
        next.auto_paste = v;
    }
    if let Some(v) = patch.backup_enabled {
        next.backup_enabled = v;
    }
    if let Some(v) = &patch.backup_dir {
        next.backup_dir = Some(v.clone());
    }
    if let Some(v) = &patch.sync_endpoint {
        next.sync_endpoint = Some(v.clone());
    }
    if let Some(v) = patch.http_receiver_enabled {
        next.http_receiver_enabled = v;
    }
    if let Some(v) = patch.local_only {
        next.local_only = v;
    }
    if let Some(v) = &patch.ring_hotkey_reverse {
        next.ring_hotkey_reverse = v.clone();
    }
    if let Some(v) = &patch.ring_hotkey_forward {
        next.ring_hotkey_forward = v.clone();
    }
    if let Some(v) = &patch.ring_hotkey_overlay {
        next.ring_hotkey_overlay = v.clone();
    }
    if let Some(v) = patch.ring_capacity {
        next.ring_capacity = v;
    }
    if let Some(v) = patch.ring_idle_dismiss_ms {
        next.ring_idle_dismiss_ms = v;
    }
    if let Some(v) = patch.ring_wrap {
        next.ring_wrap = v;
    }
    if let Some(v) = patch.ring_include_sensitive {
        next.ring_include_sensitive = v;
    }
    if let Some(v) = patch.ring_include_files {
        next.ring_include_files = v;
    }
    if let Some(v) = patch.ring_include_images {
        next.ring_include_images = v;
    }
    next
}

#[cfg(feature = "http_receiver")]
pub fn is_http_receiver_enabled(_state: &Arc<crate::state::AppState>) -> bool {
    // Implementation: read the persisted settings; inlined for feature gating simplicity.
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPatch {
    pub retention_days: Option<i64>,
    pub max_clips: Option<i64>,
    pub hotkey: Option<String>,
    pub theme: Option<ThemeMode>,
    pub autostart: Option<bool>,
    pub storage_dir: Option<PathBuf>,
    pub excluded_apps: Option<Vec<String>>,
    pub sensitive_apps: Option<Vec<String>>,
    pub auto_paste: Option<bool>,
    pub backup_enabled: Option<bool>,
    pub backup_dir: Option<PathBuf>,
    pub sync_endpoint: Option<String>,
    pub http_receiver_enabled: Option<bool>,
    pub local_only: Option<bool>,
    // -- Clipboard Ring --
    pub ring_hotkey_reverse: Option<String>,
    pub ring_hotkey_forward: Option<String>,
    pub ring_hotkey_overlay: Option<String>,
    pub ring_capacity: Option<usize>,
    pub ring_idle_dismiss_ms: Option<u64>,
    pub ring_wrap: Option<bool>,
    pub ring_include_sensitive: Option<bool>,
    pub ring_include_files: Option<bool>,
    pub ring_include_images: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
    Graphite,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
    }
}
