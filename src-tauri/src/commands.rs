//! Tauri command surface. Every function here is exposed to the frontend.

use std::sync::Arc;

use arboard::Clipboard;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
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
