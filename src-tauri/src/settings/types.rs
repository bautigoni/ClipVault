//! Persisted user settings (Tauri store).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::ThemeMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub retention_days: i64,             // 0 = infinite
    pub max_clips: i64,                  // soft cap, oldest non-favorite pruned
    pub hotkey: String,                  // serialized combo
    pub theme: ThemeMode,
    pub autostart: bool,
    pub storage_dir: Option<PathBuf>,
    pub excluded_apps: Vec<String>,      // never record from these
    pub sensitive_apps: Vec<String>,     // record hash only
    pub auto_paste: bool,
    pub backup_enabled: bool,
    pub backup_dir: Option<PathBuf>,
    /// `true` (the default) keeps ClipVault 100% on-device: the binary
    /// performs no network requests and any values in `sync_endpoint` or
    /// `http_receiver_enabled` are ignored. Flip to `false` to opt into
    /// self-hosted sync and the browser-extension HTTP receiver.
    pub local_only: bool,
    pub sync_endpoint: Option<String>,
    pub http_receiver_enabled: bool,
    // -- Clipboard Ring --
    #[serde(default = "default_ring_hotkey_reverse")]
    pub ring_hotkey_reverse: String,
    #[serde(default = "default_ring_hotkey_forward")]
    pub ring_hotkey_forward: String,
    #[serde(default = "default_ring_hotkey_overlay")]
    pub ring_hotkey_overlay: String,
    #[serde(default = "default_ring_capacity")]
    pub ring_capacity: usize,
    #[serde(default = "default_ring_idle_dismiss_ms")]
    pub ring_idle_dismiss_ms: u64,
    #[serde(default = "default_ring_wrap")]
    pub ring_wrap: bool,
    #[serde(default = "default_ring_include_sensitive")]
    pub ring_include_sensitive: bool,
    #[serde(default = "default_ring_include_files")]
    pub ring_include_files: bool,
    #[serde(default = "default_ring_include_images")]
    pub ring_include_images: bool,
    /// String inserted between clips when the user merges a multi-selection
    /// from the palette. Defaults to a single space (" ") so back-to-back
    /// copies of short words glue together as the user expects.
    #[serde(default = "default_merge_separator")]
    pub merge_separator: String,
    /// Number of rows the palette jumps when the user holds Ctrl while
    /// pressing ↑/↓. 0 = snap to top/bottom of the list.
    #[serde(default = "default_palette_jump_size")]
    pub palette_jump_size: usize,
}

fn default_ring_hotkey_reverse() -> String { "CommandOrControl+Shift+V".into() }
fn default_ring_hotkey_forward() -> String { "CommandOrControl+Shift+Alt+V".into() }
fn default_ring_hotkey_overlay() -> String { "CommandOrControl+Shift+R".into() }
fn default_ring_capacity() -> usize { 64 }
fn default_ring_idle_dismiss_ms() -> u64 { 30_000 }
fn default_ring_wrap() -> bool { true }
fn default_ring_include_sensitive() -> bool { false }
fn default_ring_include_files() -> bool { true }
fn default_ring_include_images() -> bool { true }
fn default_merge_separator() -> String { " ".into() }
fn default_palette_jump_size() -> usize { 0 }
fn default_local_only() -> bool { true }

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: 0,
            max_clips: 1_000_000,
            hotkey: "CommandOrControl+Shift+V".into(),
            theme: ThemeMode::System,
            autostart: false,
            storage_dir: None,
            excluded_apps: vec!["KeePass.exe".into(), "1password.exe".into()],
            sensitive_apps: vec![],
            auto_paste: true,
            backup_enabled: false,
            backup_dir: None,
            local_only: default_local_only(),
            sync_endpoint: None,
            http_receiver_enabled: false,
            ring_hotkey_reverse: default_ring_hotkey_reverse(),
            ring_hotkey_forward: default_ring_hotkey_forward(),
            ring_hotkey_overlay: default_ring_hotkey_overlay(),
            ring_capacity: default_ring_capacity(),
            ring_idle_dismiss_ms: default_ring_idle_dismiss_ms(),
            ring_wrap: default_ring_wrap(),
            ring_include_sensitive: default_ring_include_sensitive(),
            ring_include_files: default_ring_include_files(),
            ring_include_images: default_ring_include_images(),
            merge_separator: default_merge_separator(),
            palette_jump_size: default_palette_jump_size(),
        }
    }
}
