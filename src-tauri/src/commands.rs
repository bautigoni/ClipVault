//! Tauri command surface. Every function here is exposed to the frontend.

use std::sync::Arc;

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tracing::info;

use crate::db::repo::{self, ClipFilter, ClipPatch};
use crate::import_export::{self, ImportPolicy, ImportReport};
use crate::settings::{self, SettingsPatch};
use crate::state::{AppState, Clip, Collection, SearchPage, Snippet};

type AppStateHandle = Arc<AppState>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, AppStateHandle>,
    query: Option<String>,
    limit: Option<usize>,
    cursor: Option<String>,
    source_app: Option<String>,
    collection_id: Option<String>,
    kind: Option<String>,
    favorites_only: Option<bool>,
    pinned_only: Option<bool>,
    tag: Option<String>,
) -> Result<SearchPage, String> {
    let filter = ClipFilter {
        query,
        kind,
        source_app,
        collection_id,
        favorites_only: favorites_only.unwrap_or(false),
        pinned_only: pinned_only.unwrap_or(false),
        since: None,
        until: None,
        tag,
    };
    crate::search::search(&state.db, &filter, limit.unwrap_or(50), cursor.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn list_clips(
    state: State<'_, AppStateHandle>,
    limit: Option<usize>,
    cursor: Option<String>,
    source_app: Option<String>,
    collection_id: Option<String>,
    kind: Option<String>,
    favorites_only: Option<bool>,
    pinned_only: Option<bool>,
    tag: Option<String>,
) -> Result<SearchPage, String> {
    let filter = ClipFilter {
        query: None,
        kind,
        source_app,
        collection_id,
        favorites_only: favorites_only.unwrap_or(false),
        pinned_only: pinned_only.unwrap_or(false),
        since: None,
        until: None,
        tag,
    };
    crate::search::search(&state.db, &filter, limit.unwrap_or(100), cursor.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn get_clip(state: State<'_, AppStateHandle>, id: String) -> Result<Option<Clip>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::get_clip(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppStateHandle>,
    id: String,
    value: bool,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::apply_patch(&conn, &id, &ClipPatch { is_favorite: Some(value), ..Default::default() }).map_err(err)
}

#[tauri::command]
pub async fn pin_clip(state: State<'_, AppStateHandle>, id: String, pinned: bool) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::apply_patch(&conn, &id, &ClipPatch { is_pinned: Some(pinned), ..Default::default() }).map_err(err)
}

#[tauri::command]
pub async fn delete_clip(state: State<'_, AppStateHandle>, id: String) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::delete_clip(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn delete_clips(state: State<'_, AppStateHandle>, ids: Vec<String>) -> Result<usize, String> {
    let conn = state.db.get().map_err(err)?;
    let mut count = 0;
    for id in ids {
        repo::delete_clip(&conn, &id).map_err(err)?;
        count += 1;
    }
    Ok(count)
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppStateHandle>) -> Result<usize, String> {
    let conn = state.db.get().map_err(err)?;
    repo::clear_history(&conn).map_err(err)
}

#[tauri::command]
pub async fn update_clip_meta(
    state: State<'_, AppStateHandle>,
    id: String,
    patch: ClipPatch,
) -> Result<Clip, String> {
    let conn = state.db.get().map_err(err)?;
    repo::apply_patch(&conn, &id, &patch).map_err(err)?;
    repo::get_clip(&conn, &id)
        .map_err(err)?
        .ok_or_else(|| "clip disappeared".to_string())
}

#[tauri::command]
pub async fn list_collections(state: State<'_, AppStateHandle>) -> Result<Vec<Collection>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::list_collections(&conn).map_err(err)
}

#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppStateHandle>,
    name: String,
    icon: Option<String>,
) -> Result<Collection, String> {
    let conn = state.db.get().map_err(err)?;
    repo::create_collection(&conn, &name, icon.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, AppStateHandle>, id: String) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::delete_collection(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn rename_collection(
    state: State<'_, AppStateHandle>,
    id: String,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::rename_collection(&conn, &id, &name, icon.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn assign_to_collection(
    state: State<'_, AppStateHandle>,
    clip_id: String,
    collection_id: Option<String>,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::apply_patch(
        &conn,
        &clip_id,
        &ClipPatch { collection_id: Some(collection_id), ..Default::default() },
    )
    .map_err(err)
}

#[tauri::command]
pub async fn list_snippets(state: State<'_, AppStateHandle>) -> Result<Vec<Snippet>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::list_snippets(&conn).map_err(err)
}

#[tauri::command]
pub async fn search_snippets(
    state: State<'_, AppStateHandle>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<Snippet>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::search_snippets(&conn, &query, limit.unwrap_or(100)).map_err(err)
}

#[derive(Debug, Deserialize)]
pub struct SnippetInput {
    pub id: Option<String>,
    pub title: String,
    pub language: String,
    pub body: String,
    pub is_favorite: bool,
}

#[tauri::command]
pub async fn upsert_snippet(
    state: State<'_, AppStateHandle>,
    input: SnippetInput,
) -> Result<Snippet, String> {
    let conn = state.db.get().map_err(err)?;
    repo::upsert_snippet(
        &conn,
        input.id.as_deref(),
        &input.title,
        &input.language,
        &input.body,
        input.is_favorite,
    )
    .map_err(err)
}

#[tauri::command]
pub async fn delete_snippet(state: State<'_, AppStateHandle>, id: String) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::delete_snippet(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn copy_snippet_to_clipboard(
    state: State<'_, AppStateHandle>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    let body = repo::get_snippet_body(&conn, &id).map_err(err)?;
    if let Some(text) = body {
        let mut clipboard = Clipboard::new().map_err(err)?;
        let _ = clipboard.clear();
        clipboard.set_text(text).map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn read_image_full(
    state: State<'_, AppStateHandle>,
    rel_path: String,
) -> Result<Vec<u8>, String> {
    crate::images::storage::load_full(&state.data_dir, &rel_path).map_err(err)
}

#[tauri::command]
pub async fn read_image_thumb(
    state: State<'_, AppStateHandle>,
    rel_path: String,
) -> Result<Vec<u8>, String> {
    crate::images::storage::load_thumb(&state.data_dir, &rel_path).map_err(err)
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub path: String,
}

#[tauri::command]
pub async fn export_db(state: State<'_, AppStateHandle>, req: ExportRequest) -> Result<(), String> {
    let archive_path = std::path::PathBuf::from(&req.path);
    let db_path = state.data_dir.join("clipvault.db");
    let images_dir = state.data_dir.join("images");
    let thumbs_dir = state.data_dir.join("thumbs");
    import_export::export_to_zip(
        &archive_path,
        &db_path,
        Some(&images_dir),
        Some(&thumbs_dir),
    )
    .map_err(err)
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub path: String,
    pub policy: ImportPolicy,
}

#[tauri::command]
pub async fn import_db(state: State<'_, AppStateHandle>, req: ImportRequest) -> Result<ImportReport, String> {
    let archive_path = std::path::PathBuf::from(&req.path);
    import_export::import_from_zip(&archive_path, &state.data_dir, req.policy).map_err(err)
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<settings::Settings, String> {
    Ok(settings::load_settings(&app))
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    patch: SettingsPatch,
) -> Result<settings::Settings, String> {
    let current = settings::load_settings(&app);
    let next = settings::merge_settings(&current, &patch);
    // Guard: when `local_only` is on, sync and the browser-extension HTTP
    // receiver are hard-disabled. This is a command-level (not just UI-level)
    // safeguard so the binary cannot be coerced into making network calls
    // even if a frontend bypass is attempted.
    if next.local_only {
        if next.sync_endpoint.is_some() || next.http_receiver_enabled {
            let next = settings::Settings {
                sync_endpoint: None,
                http_receiver_enabled: false,
                ..next
            };
            settings::save_settings(&app, &next).map_err(err)?;
            return apply_settings_side_effects(app, state, patch, next);
        }
    }
    settings::save_settings(&app, &next).map_err(err)?;
    apply_settings_side_effects(app, state, patch, next)
}

/// Apply the runtime side-effects of a settings change (autostart, hotkey,
/// retention). Split out so the `local_only` guard can reuse the same
/// post-save logic without duplicating it.
fn apply_settings_side_effects(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    patch: SettingsPatch,
    next: settings::Settings,
) -> Result<settings::Settings, String> {
    // Apply side-effects: autostart, hotkey
    if let Some(autostart) = patch.autostart {
        let mgr = app.autolaunch();
        if autostart {
            mgr.enable().map_err(err)?;
        } else {
            mgr.disable().map_err(err)?;
        }
    }
    if let Some(combo) = &patch.hotkey {
        crate::hotkey::warn_if_invalid(combo);
    }
    // Trigger retention run if retention changed
    let _ = crate::settings::retention::run_once(&state);
    Ok(next)
}

#[tauri::command]
pub async fn show_palette(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("palette") {
        window.show().map_err(err)?;
        window.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_palette(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("palette") {
        window.hide().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_main(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(err)?;
        window.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_main(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn register_hotkey(app: AppHandle, combo: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    // Unregister all first
    let _ = app.global_shortcut().unregister_all();
    crate::hotkey::register(&app, &combo)?;
    // Re-register the alt palette hotkey (Win+V by default) so it survives
    // a user-initiated hotkey reconfigure.
    if let Err(e) = crate::hotkey::register(&app, "Win+V") {
        tracing::warn!(?e, combo = "Win+V", "failed to re-register Win+V hotkey");
    }

    // Re-register the additional ring hotkeys (forward / overlay) so they
    // survive a user-initiated hotkey reconfigure. Without this, switching
    // the palette hotkey would silently disable the ring's other hotkeys.
    let s = crate::settings::load_settings(&app);
    if let Err(e) = crate::hotkey::register_ring_forward(&app, &s.ring_hotkey_forward) {
        tracing::warn!(?e, combo = s.ring_hotkey_forward, "failed to re-register ring forward hotkey");
    }
    if let Err(e) = crate::hotkey::register_ring_overlay(&app, &s.ring_hotkey_overlay) {
        tracing::warn!(?e, combo = s.ring_hotkey_overlay, "failed to re-register ring overlay hotkey");
    }
    Ok(())
}

#[tauri::command]
pub async fn copy_clip_to_clipboard(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    let clip = repo::get_clip(&conn, &id).map_err(err)?.ok_or_else(|| "clip not found".to_string())?;
    let hash = clip.content_hash.clone();
    let mut clipboard = Clipboard::new().map_err(err)?;
    // Always start by clearing so partial / wrong-format data doesn't linger.
    let _ = clipboard.clear();
    match clip.kind.as_str() {
        "text" | "url" => {
            let text = clip.text_preview.clone().unwrap_or_default();
            clipboard.set_text(text).map_err(err)?;
        }
        "image" => {
            if let Some(meta) = &clip.image {
                let bytes = crate::images::storage::load_full(&state.data_dir, &meta.path).map_err(err)?;
                let img = image::load_from_memory(&bytes).map_err(err)?;
                let rgba = img.to_rgba8();
                let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                let copied = arboard::ImageData {
                    width: w,
                    height: h,
                    bytes: rgba.into_raw().into(),
                };
                clipboard.set_image(copied).map_err(err)?;
            }
        }
        "files" => {
            if let Some(paths) = &clip.file_paths {
                crate::clipboard::files::write_file_list(paths).map_err(err)?;
            }
        }
        _ => return Err(format!("unsupported clip kind {}", clip.kind)),
    }
    crate::clipboard::watcher::arm_suppress(&state, hash);
    info!(id, "clip copied to clipboard");

    // Optional auto-paste: simulate Ctrl+V into the previously focused window
    // so the user doesn't have to press it themselves. Driven by the
    // `auto_paste` setting; disabled by default. Runs in a background thread
    // so the IPC response is not blocked by the ~120ms synthetic keypress.
    let settings = crate::settings::load_settings(&app);
    if settings.auto_paste {
        std::thread::spawn(|| {
            // Short delay so the receiving app's clipboard observer has time
            // to see the new content before the paste key fires.
            std::thread::sleep(std::time::Duration::from_millis(60));
            crate::paste::send_ctrl_v();
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn merge_and_paste_clips(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    ids: Vec<String>,
) -> Result<String, String> {
    if ids.is_empty() {
        return Err("no clips selected".to_string());
    }
    if ids.len() > 32 {
        return Err("too many clips (max 32)".to_string());
    }
    let conn = state.db.get().map_err(err)?;
    // Resolve clips in selection order; reject anything non-text so the user
    // doesn't get a half-broken paste (e.g. an image blob concatenated with text).
    let mut pieces: Vec<String> = Vec::with_capacity(ids.len());
    let mut last_hash: Option<String> = None;
    for id in &ids {
        let clip = repo::get_clip(&conn, id)
            .map_err(err)?
            .ok_or_else(|| format!("clip {} not found", id))?;
        match clip.kind.as_str() {
            "text" | "url" => {
                let text = clip.text_preview.clone().unwrap_or_default();
                if !text.is_empty() {
                    pieces.push(text);
                }
                last_hash = Some(clip.content_hash.clone());
            }
            other => {
                return Err(format!(
                    "clip {} is a {} — only text/url clips can be merged",
                    id, other
                ));
            }
        }
    }
    drop(conn);

    // Separator: configurable via settings.merge_separator. Defaults to a
    // single space, which is what users usually want when they "combine
    // copies" of short text. We trim trailing whitespace from each piece
    // so back-to-back copies don't leave stray indentation from IDEs.
    let settings = crate::settings::load_settings(&app);
    let separator = settings.merge_separator.clone();
    let merged = pieces
        .iter()
        .map(|p| p.trim_end().to_string())
        .collect::<Vec<_>>()
        .join(&separator);

    let mut clipboard = Clipboard::new().map_err(err)?;
    let _ = clipboard.clear();
    clipboard.set_text(merged.clone()).map_err(err)?;
    if let Some(h) = last_hash {
        crate::clipboard::watcher::arm_suppress(&state, h);
    }
    info!(count = ids.len(), bytes = merged.len(), "merged clips copied to clipboard");

    // Auto-paste: respect the same setting as the single-clip path. The
    // receive window loses focus when we open the palette, so this is the
    // window that was focused *before* the palette was opened.
    let settings = crate::settings::load_settings(&app);
    if settings.auto_paste {
        let app_for_paste = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(60));
            crate::paste::send_ctrl_v();
            // Tiny log so we can confirm in the trace that the merge paste ran.
            tracing::info!(app = ?app_for_paste.config().identifier, "merge auto-paste dispatched");
        });
    }

    Ok(merged)
}

#[tauri::command]
pub async fn list_source_apps(state: State<'_, AppStateHandle>) -> Result<Vec<(String, i64)>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::list_source_apps(&conn).map_err(err)
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppStateHandle>) -> Result<Vec<String>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::list_tags(&conn).map_err(err)
}

#[tauri::command]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(err)?;
    } else {
        mgr.disable().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn run_backup(state: State<'_, AppStateHandle>) -> Result<String, String> {
    crate::settings::backup::run_backup(&state).map_err(err)?;
    let path = state.data_dir.join("backups");
    Ok(path.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// Clipboard Ring commands
// ---------------------------------------------------------------------------

fn ring_controller(app: &AppHandle) -> Result<crate::ring::controller::RingController, String> {
    use tauri::Manager;
    Ok(app.state::<crate::ring::controller::RingController>().inner().clone())
}

#[derive(Debug, Deserialize)]
pub struct RingScopeInput {
    pub kind: String,                // "global" | "favorites" | "collection" | "application" | "kind" | "named_set"
    pub collection_id: Option<String>,
    pub application_exe: Option<String>,
    pub clip_kind: Option<String>,
    pub set_id: Option<String>,
}

fn parse_scope(input: RingScopeInput) -> crate::ring::buffer::RingScope {
    use crate::ring::buffer::RingScope;
    match input.kind.as_str() {
        "favorites" => RingScope::Favorites,
        "collection" => RingScope::Collection(input.collection_id.unwrap_or_default()),
        "application" => RingScope::Application { exe: input.application_exe.unwrap_or_default() },
        "kind" => RingScope::Kind(input.clip_kind.unwrap_or_else(|| "text".into())),
        "named_set" => RingScope::NamedSet(input.set_id.unwrap_or_default()),
        _ => RingScope::Global,
    }
}

#[derive(Debug, Deserialize)]
pub struct RingConfigInput {
    pub capacity: Option<usize>,
    pub wrap: Option<bool>,
    pub idle_dismiss_ms: Option<u64>,
    pub include_sensitive: Option<bool>,
    pub include_files: Option<bool>,
    pub include_images: Option<bool>,
}

fn merge_config(base: &crate::ring::buffer::RingConfig, input: RingConfigInput) -> crate::ring::buffer::RingConfig {
    crate::ring::buffer::RingConfig {
        capacity: input.capacity.unwrap_or(base.capacity),
        wrap: input.wrap.unwrap_or(base.wrap),
        idle_dismiss_ms: input.idle_dismiss_ms.unwrap_or(base.idle_dismiss_ms),
        include_sensitive: input.include_sensitive.unwrap_or(base.include_sensitive),
        include_files: input.include_files.unwrap_or(base.include_files),
        include_images: input.include_images.unwrap_or(base.include_images),
    }
}

#[tauri::command]
pub async fn ring_set_scope(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    scope: RingScopeInput,
    config: Option<RingConfigInput>,
) -> Result<usize, String> {
    let ring = ring_controller(&app)?;
    let parsed = parse_scope(scope);
    let cfg = merge_config(&ring.lock().config, config.unwrap_or(RingConfigInput {
        capacity: None,
        wrap: None,
        idle_dismiss_ms: None,
        include_sensitive: None,
        include_files: None,
        include_images: None,
    }));
    ring.set_scope(&app, &state, parsed, cfg);
    let count = ring.lock().slot_count();
    Ok(count)
}

#[tauri::command]
pub async fn ring_reverse(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
) -> Result<serde_json::Value, String> {
    let ring = ring_controller(&app)?;
    let r = ring.reverse(&app, &state);
    serde_json::to_value(&r).map_err(err)
}

#[tauri::command]
pub async fn ring_forward(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
) -> Result<serde_json::Value, String> {
    let ring = ring_controller(&app)?;
    let r = ring.forward(&app, &state);
    serde_json::to_value(&r).map_err(err)
}

#[tauri::command]
pub async fn ring_jump(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    index: usize,
) -> Result<serde_json::Value, String> {
    let ring = ring_controller(&app)?;
    let r = ring.jump(&app, &state, index);
    serde_json::to_value(&r).map_err(err)
}

#[tauri::command]
pub async fn ring_dismiss(app: AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    let ring = ring_controller(&app)?;
    ring.dismiss();
    // Tell the frontend to hide the overlay / clear its active state.
    let _ = app.emit("clip://ring-dismissed", serde_json::json!({}));
    Ok(())
}

#[tauri::command]
pub async fn ring_is_active(app: AppHandle) -> Result<bool, String> {
    let ring = ring_controller(&app)?;
    Ok(ring.is_active())
}

#[tauri::command]
pub async fn ring_preview(app: AppHandle, n: Option<usize>) -> Result<Vec<crate::ring::buffer::RingSlotView>, String> {
    let ring = ring_controller(&app)?;
    Ok(ring.preview(n.unwrap_or(5)))
}

#[tauri::command]
pub async fn ring_list_sets(state: State<'_, AppStateHandle>) -> Result<Vec<repo::RingSet>, String> {
    repo::list_ring_sets(&state).map_err(err)
}

#[tauri::command]
pub async fn ring_create_set(
    state: State<'_, AppStateHandle>,
    name: String,
    scope_kind: String,
    scope_ref: Option<String>,
) -> Result<repo::RingSet, String> {
    repo::create_ring_set(&state, &name, &scope_kind, scope_ref.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn ring_delete_set(state: State<'_, AppStateHandle>, id: String) -> Result<(), String> {
    repo::delete_ring_set(&state, &id).map_err(err)
}

#[tauri::command]
pub async fn ring_add_to_set(
    state: State<'_, AppStateHandle>,
    set_id: String,
    clip_id: String,
    position: Option<i64>,
) -> Result<(), String> {
    repo::add_to_ring_set(&state, &set_id, &clip_id, position.unwrap_or(0)).map_err(err)
}

#[tauri::command]
pub async fn ring_remove_from_set(
    state: State<'_, AppStateHandle>,
    set_id: String,
    clip_id: String,
) -> Result<(), String> {
    repo::remove_from_ring_set(&state, &set_id, &clip_id).map_err(err)
}

/// Diagnostic helper: triggers the same SendInput sequence that the auto-paste
/// path uses, without touching the clipboard. Useful for verifying that the
/// paste module is being invoked at all and that `SendInput` reaches the
/// foreground app. The trace is written to %APPDATA%\com.clipvault.app\debug.log.
#[tauri::command]
pub async fn test_paste() -> Result<(), String> {
    crate::paste::send_ctrl_v();
    Ok(())
}

/// Open the Windows 10/11 screen-snipping tool (the same UI as `Win+Shift+S`).
/// The captured region lands in the clipboard, so the watcher records it
/// automatically as a new image clip — no special-casing needed. We just
/// inject the chord and return; the user is in control of what gets snipped.
#[tauri::command]
pub async fn trigger_screenshot() -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
        VK_LWIN, VK_S, VK_SHIFT,
    };

    /// Tiny builder so we don't repeat the union dance six times.
    fn keybd(vk: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { Default::default() },
                    ..Default::default()
                },
            },
        }
    }

    let downs = [
        keybd(VK_LWIN, false),
        keybd(VK_SHIFT, false),
        keybd(VK_S, false),
    ];
    let ups = [
        keybd(VK_S, true),
        keybd(VK_SHIFT, true),
        keybd(VK_LWIN, true),
    ];
    unsafe {
        SendInput(&downs, std::mem::size_of::<INPUT>() as i32);
        SendInput(&ups, std::mem::size_of::<INPUT>() as i32);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Text transformations
// ---------------------------------------------------------------------------
//
// Cheap, pure-function operations on text. They take a string + a transform
// kind and return the transformed string. The frontend calls them via the
// `transformClip` command and shows a small menu of options next to each
// text clip in the palette. The result is *not* auto-pasted — it lands in
// the clipboard and the user presses Enter again to paste, so we never
// surprise them with a paste they didn't ask for.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextTransform {
    Uppercase,
    Lowercase,
    TitleCase,
    SentenceCase,
    Trim,
    CollapseWhitespace,
    DedupLines,
    SortLinesAsc,
    SortLinesDesc,
    UniqueLines,
    Reverse,
    StripEmptyLines,
    ToSingleLine,
    UrlEncode,
    UrlDecode,
    Base64Encode,
    Base64Decode,
    Count,
}

impl TextTransform {
    fn label(&self) -> &'static str {
        match self {
            TextTransform::Uppercase => "UPPERCASE",
            TextTransform::Lowercase => "lowercase",
            TextTransform::TitleCase => "Title Case",
            TextTransform::SentenceCase => "Sentence case",
            TextTransform::Trim => "Trim",
            TextTransform::CollapseWhitespace => "Collapse whitespace",
            TextTransform::DedupLines => "Dedupe lines",
            TextTransform::SortLinesAsc => "Sort A→Z",
            TextTransform::SortLinesDesc => "Sort Z→A",
            TextTransform::UniqueLines => "Unique lines",
            TextTransform::Reverse => "Reverse",
            TextTransform::StripEmptyLines => "Strip empty lines",
            TextTransform::ToSingleLine => "To single line",
            TextTransform::UrlEncode => "URL encode",
            TextTransform::UrlDecode => "URL decode",
            TextTransform::Base64Encode => "Base64 encode",
            TextTransform::Base64Decode => "Base64 decode",
            TextTransform::Count => "Count",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransformRequest {
    pub text: String,
    pub kind: TextTransform,
}

#[derive(Debug, Serialize)]
pub struct TransformResult {
    /// The transformed text. For the `Count` kind this is empty.
    pub text: String,
    /// A short, human-readable label of what was applied.
    pub label: String,
    /// Length of the input in characters, returned so the UI can show
    /// "1234 chars → 1234 chars" or "200 → 80" for the count kind.
    pub in_len: usize,
    /// Length of the output. Same as in_len for the Count kind.
    pub out_len: usize,
    /// For the Count kind, the structured count payload (chars, words, lines).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<Counts>,
}

#[derive(Debug, Serialize)]
pub struct Counts {
    pub chars: usize,
    pub words: usize,
    pub lines: usize,
    pub bytes: usize,
}

fn apply_transform(text: &str, kind: &TextTransform) -> String {
    use TextTransform::*;
    match kind {
        Uppercase => text.to_uppercase(),
        Lowercase => text.to_lowercase(),
        TitleCase => title_case(text),
        SentenceCase => sentence_case(text),
        Trim => text.trim().to_string(),
        CollapseWhitespace => collapse_whitespace(text),
        DedupLines => dedup_lines(text, false),
        UniqueLines => dedup_lines(text, true),
        SortLinesAsc => sort_lines(text, false),
        SortLinesDesc => sort_lines(text, true),
        Reverse => text.chars().rev().collect(),
        StripEmptyLines => text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        ToSingleLine => text.split_whitespace().collect::<Vec<_>>().join(" "),
        UrlEncode => url_encode(text),
        UrlDecode => url_decode(text),
        Base64Encode => base64_encode(text),
        Base64Decode => base64_decode(text),
        Count => String::new(),
    }
}

fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for c in s.chars() {
        if c.is_whitespace() {
            at_word_start = true;
            out.push(c);
        } else if at_word_start {
            for u in c.to_uppercase() {
                out.push(u);
            }
            at_word_start = false;
        } else {
            for u in c.to_lowercase() {
                out.push(u);
            }
        }
    }
    out
}

fn sentence_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_sentence_start = true;
    for c in s.chars() {
        if matches!(c, '.' | '!' | '?' | '\n') {
            at_sentence_start = true;
            for u in c.to_lowercase() {
                out.push(u);
            }
        } else if at_sentence_start && c.is_alphabetic() {
            for u in c.to_uppercase() {
                out.push(u);
            }
            at_sentence_start = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    out
}

fn dedup_lines(s: &str, unique_only: bool) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        if unique_only {
            if seen.insert(line.to_string()) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(line);
            }
        } else {
            if !seen.insert(line.to_string()) {
                continue;
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
        }
    }
    out
}

fn sort_lines(s: &str, reverse: bool) -> String {
    let mut lines: Vec<&str> = s.lines().collect();
    lines.sort_by(|a, b| {
        if reverse {
            b.cmp(a)
        } else {
            a.cmp(b)
        }
    });
    let mut out = String::with_capacity(s.len());
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(l);
    }
    out
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// base64 uses an external crate, but we can implement a minimal
// implementation here without adding a dep. Kept simple — no streaming,
// no URL-safe variants, no multi-line output.
const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = (bytes[i] as u32) << 16;
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
        out.push(B64_ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

fn b64_val(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn base64_decode(s: &str) -> String {
    // Strip padding for length math, then iterate over 4-char groups.
    let trimmed: Vec<u8> = s.bytes().filter(|b| *b != b'=' && !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(trimmed.len() * 3 / 4);
    let mut i = 0;
    while i + 4 <= trimmed.len() {
        let v0 = b64_val(trimmed[i]).unwrap_or(0);
        let v1 = b64_val(trimmed[i + 1]).unwrap_or(0);
        let v2 = b64_val(trimmed[i + 2]).unwrap_or(0);
        let v3 = b64_val(trimmed[i + 3]).unwrap_or(0);
        let n = ((v0 as u32) << 18) | ((v1 as u32) << 12) | ((v2 as u32) << 6) | (v3 as u32);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
        out.push(n as u8);
        i += 4;
    }
    let rem = trimmed.len() - i;
    if rem == 2 {
        let v0 = b64_val(trimmed[i]).unwrap_or(0);
        let v1 = b64_val(trimmed[i + 1]).unwrap_or(0);
        let n = ((v0 as u32) << 18) | ((v1 as u32) << 12);
        out.push((n >> 16) as u8);
    } else if rem == 3 {
        let v0 = b64_val(trimmed[i]).unwrap_or(0);
        let v1 = b64_val(trimmed[i + 1]).unwrap_or(0);
        let v2 = b64_val(trimmed[i + 2]).unwrap_or(0);
        let n = ((v0 as u32) << 18) | ((v1 as u32) << 12) | ((v2 as u32) << 6);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[tauri::command]
pub async fn transform_clip(req: TransformRequest) -> Result<TransformResult, String> {
    let in_len = req.text.chars().count();
    let label = req.kind.label().to_string();
    let counts = if matches!(req.kind, TextTransform::Count) {
        Some(Counts {
            chars: in_len,
            words: req.text.split_whitespace().count(),
            lines: req.text.lines().count(),
            bytes: req.text.len(),
        })
    } else {
        None
    };
    let out = apply_transform(&req.text, &req.kind);
    let out_len = out.chars().count();
    Ok(TransformResult {
        text: out,
        label,
        in_len,
        out_len,
        counts,
    })
}

// ---------------------------------------------------------------------------
// Drag-and-drop file ingest
// ---------------------------------------------------------------------------
//
// The main window's file drop handler in the frontend calls this command with
// the list of file paths the user dragged onto the app. We register a single
// `files` clip with all paths attached, and emit `clip://updated` so the
// palette refreshes itself. Identical to what `record_files` does for native
// clipboard file drops, but callable from a UI gesture.

#[tauri::command]
pub async fn ingest_dropped_files(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
    paths: Vec<String>,
) -> Result<String, String> {
    if paths.is_empty() {
        return Err("no paths provided".into());
    }
    // Filter out anything that doesn't exist on disk — Tauri gives us the
    // raw paths, but a user could have moved the file mid-drag or the path
    // could be junk. We keep the empty-list as an error so the UI can show
    // a "couldn't import" toast.
    let valid: Vec<String> = paths
        .into_iter()
        .filter(|p| std::path::Path::new(p).exists())
        .collect();
    if valid.is_empty() {
        return Err("none of the dropped paths exist on disk".into());
    }

    let now = repo::now_ms();
    let json = serde_json::to_string(&valid).map_err(err)?;
    let total_size: i64 = valid
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len() as i64)
        .sum();
    let preview = valid.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
    let hash = blake3::hash(json.as_bytes()).to_hex().to_string();

    let conn = state.db.get().map_err(err)?;
    if let Some(existing) = repo::find_recent_duplicate(
        &conn,
        &hash,
        crate::clipboard::DEDUPE_WINDOW_MS,
        now,
    )
    .map_err(err)?
    {
        repo::bump_usage(&conn, &existing, now).map_err(err)?;
        let _ = app.emit("clip://updated", serde_json::json!({ "id": existing }));
        return Ok(existing);
    }
    let id = repo::insert_clip(
        &conn,
        "files",
        &hash,
        Some(&preview),
        total_size,
        Some("drop"),
        None,
        now,
        &repo::resolve_active_user(&conn, settings::load_settings(&app).active_user_id.as_deref())
            .map_err(err)?,
    )
    .map_err(err)?;
    repo::attach_files(&conn, &id, &json).map_err(err)?;
    drop(conn);
    let _ = app.emit("clip://updated", serde_json::json!({ "id": id }));
    Ok(id)
}

// ---------------------------------------------------------------------------
// OCR on image clips
// ---------------------------------------------------------------------------
//
// Runs the Windows built-in OCR engine over the image bytes of a stored
// image clip, then writes the recognized text into the `clip_ocr` table
// so the frontend can show a "Text in image" tab. The watcher calls this
// in the background right after `record_image` returns.

#[tauri::command]
pub async fn ocr_clip(
    state: State<'_, AppStateHandle>,
    clip_id: String,
) -> Result<Option<String>, String> {
    let conn = state.db.get().map_err(err)?;
    let rel: Option<String> = conn
        .query_row(
            "SELECT path FROM images WHERE clip_id = ?",
            rusqlite::params![clip_id],
            |r| r.get(0),
        )
        .ok();
    let Some(rel) = rel else {
        return Ok(None);
    };
    drop(conn);
    let bytes = crate::images::storage::load_full(&state.data_dir, &rel).map_err(err)?;
    match crate::ocr::recognize(&bytes) {
        Ok(text) if !text.trim().is_empty() => {
            let conn = state.db.get().map_err(err)?;
            crate::db::repo::save_ocr(&conn, &clip_id, &text, repo::now_ms()).map_err(err)?;
            Ok(Some(text))
        }
        Ok(_) => Ok(None),
        Err(e) => {
            tracing::warn!(?e, "ocr_clip: engine returned no text");
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn ocr_get(
    state: State<'_, AppStateHandle>,
    clip_id: String,
) -> Result<Option<String>, String> {
    let conn = state.db.get().map_err(err)?;
    crate::db::repo::load_ocr(&conn, &clip_id).map_err(err)
}

// ---------------------------------------------------------------------------
// Activity log
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_activity(
    state: State<'_, AppStateHandle>,
    limit: Option<i64>,
) -> Result<Vec<repo::ActivityEntry>, String> {
    let conn = state.db.get().map_err(err)?;
    repo::list_activity(&conn, limit.unwrap_or(200).clamp(1, 1000)).map_err(err)
}

#[tauri::command]
pub async fn clear_activity(state: State<'_, AppStateHandle>) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::clear_activity(&conn).map_err(err)
}

// ---------------------------------------------------------------------------
// Per-device multi-user (Phase 6 / Fase 1)
// ---------------------------------------------------------------------------
//
// The active user is persisted in `Settings.active_user_id`. The watcher
// and the list queries read that setting through `load_settings` and pass
// the resolved id down to the repo. We resolve lazily on every command
// (not once at boot) so a UI switch takes effect on the next call without
// needing to restart the app or rebind any handle.

#[tauri::command]
pub async fn users_list(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
) -> Result<Vec<repo::User>, String> {
    let conn = state.db.get().map_err(err)?;
    let users = repo::list_users(&conn).map_err(err)?;
    // Touch settings so a fresh install that has never queried the user
    // table still gets a deterministic active id.
    let _ = settings::load_settings(&app);
    Ok(users)
}

#[tauri::command]
pub async fn users_create(
    state: State<'_, AppStateHandle>,
    display_name: String,
    email: Option<String>,
) -> Result<repo::User, String> {
    let conn = state.db.get().map_err(err)?;
    repo::create_user(&conn, &display_name, email.as_deref()).map_err(err)
}

#[tauri::command]
pub async fn users_rename(
    state: State<'_, AppStateHandle>,
    id: String,
    display_name: String,
) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::rename_user(&conn, &id, &display_name).map_err(err)
}

#[tauri::command]
pub async fn users_set_default(
    state: State<'_, AppStateHandle>,
    id: String,
) -> Result<repo::User, String> {
    let conn = state.db.get().map_err(err)?;
    repo::set_default_user(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn users_delete(state: State<'_, AppStateHandle>, id: String) -> Result<(), String> {
    let conn = state.db.get().map_err(err)?;
    repo::delete_user(&conn, &id).map_err(err)
}

#[tauri::command]
pub async fn users_get_active(
    app: AppHandle,
    state: State<'_, AppStateHandle>,
) -> Result<repo::User, String> {
    let settings = settings::load_settings(&app);
    let conn = state.db.get().map_err(err)?;
    let id = repo::resolve_active_user(&conn, settings.active_user_id.as_deref())
        .map_err(err)?;
    // The resolver always returns Some unless the DB is genuinely corrupt.
    repo::get_user(&conn, &id)
        .map_err(err)?
        .ok_or_else(|| "active user not found after resolution".to_string())
}

#[tauri::command]
pub async fn users_set_active(
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    // Validate that the id exists before persisting it, so the UI cannot
    // accidentally write a dangling reference.
    let conn = app
        .state::<AppStateHandle>()
        .db
        .get()
        .map_err(err)?;
    if repo::get_user(&conn, &id).map_err(err)?.is_none() {
        return Err(format!("user {id} does not exist"));
    }
    drop(conn);
    // We don't reuse `commands::update_settings` here because it is itself
    // a Tauri command (cannot be called from inside another command) and
    // because we want the active-user write to NOT trigger the
    // `apply_settings_side_effects` fanout — switching profiles is cheap
    // and only affects what gets recorded, not any registered resources.
    let current = settings::load_settings(&app);
    let next = settings::Settings {
        active_user_id: Some(id),
        ..current
    };
    settings::save_settings(&app, &next).map_err(err)
}
