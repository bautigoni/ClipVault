//! ClipVault backend library (force rebuild after frontend dist change).
//!
//! The library is structured as a set of focused modules wired together by
//! [`run`]. The binary crate (`src/main.rs`) is a thin shim that calls `run`.

pub mod clipboard;
pub mod commands;
pub mod db;
pub mod hotkey;
pub mod images;
pub mod import_export;
pub mod paste;
pub mod ring;
pub mod search;
pub mod settings;
pub mod snippets;
pub mod state;
pub mod tray;

#[cfg(feature = "ocr")]
pub mod ocr;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "http_receiver")]
pub mod http_receiver;

use std::sync::Arc;

use tauri::{Manager, WindowEvent};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

/// Build the Tauri application with all plugins and the application state.
pub fn build_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        // Single instance: launching ClipVault again just focuses the existing window.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Tracing. We initialise a subscriber that writes to a file
            // (set by main.rs via CLIPVAULT_LOG_FILE) in addition to the
            // console, so auto-paste / watcher / hotkey traces are visible
            // in release where no console is attached.
            use tracing_subscriber::layer::SubscriberExt;
            use tracing_subscriber::util::SubscriberInitExt;

            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("clipvault=info,warn"));

            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(std::io::stderr);

            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer);

            if let Ok(log_path) = std::env::var("CLIPVAULT_LOG_FILE") {
                if let Ok(file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    let file_layer = tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .with_writer(file)
                        .with_ansi(false);
                    let _ = registry.with(file_layer).try_init();
                } else {
                    let _ = registry.try_init();
                }
            } else {
                let _ = registry.try_init();
            }

            info!("ClipVault starting up");

            // Resolve data directory (portable mode uses the executable's folder instead).
            let data_dir = settings::data_dir(app.handle())?;
            std::fs::create_dir_all(&data_dir).ok();

            // Open the database and build the shared state.
            let pool = db::open_pool(&data_dir.join("clipvault.db"))?;
            db::migrations::run(&pool)?;
            let state = AppState::new(pool, data_dir.clone(), app.handle().clone());
            app.manage(Arc::new(state.clone()));

            // Register the default hotkey through `register_hotkey` so the per-shortcut
            // handler (`hotkey::register`) is the only place that toggles the palette.
            // Doing `app.global_shortcut().register(...)` here would also trigger the
            // global `with_handler` above, causing the palette to toggle twice.
            let default_combo = "Ctrl+Shift+V";
            if let Err(e) = crate::hotkey::register(&app.handle(), default_combo) {
                tracing::warn!(?e, "failed to register default hotkey");
            }

            // Also register Win+V as a second way to open the palette. This is
            // the same combo Windows uses for its built-in clipboard history,
            // but ClipVault will still win the global shortcut race when the
            // user opts into our installer. A failure here is non-fatal: the
            // user can still open the palette with Ctrl+Shift+V.
            let alt_combo = "Win+V";
            if let Err(e) = crate::hotkey::register(&app.handle(), alt_combo) {
                tracing::warn!(?e, combo = alt_combo, "failed to register alt hotkey");
            }

            // Register the additional ring hotkeys (forward / overlay). The
            // forward hotkey is `Ctrl+Shift+Alt+V` and the overlay toggle is
            // `Ctrl+Shift+R` by default. Failures here are non-fatal — the
            // ring can still be driven via the palette/reverse hotkey.
            let settings = settings::load_settings(app.handle());
            if let Err(e) = crate::hotkey::register_ring_forward(&app.handle(), &settings.ring_hotkey_forward) {
                tracing::warn!(?e, combo = settings.ring_hotkey_forward, "failed to register ring forward hotkey");
            }
            if let Err(e) = crate::hotkey::register_ring_overlay(&app.handle(), &settings.ring_hotkey_overlay) {
                tracing::warn!(?e, combo = settings.ring_hotkey_overlay, "failed to register ring overlay hotkey");
            }

            // Start the clipboard watcher.
            clipboard::watcher::start(app.handle().clone(), state.clone());

            // Start the retention sweeper.
            settings::retention::start_sweeper(app.handle().clone(), state.clone());

            // Initialize the Clipboard Ring subsystem. The controller listens
            // to the watcher's `clip://created` and `clip://updated` events so
            // it can invalidate the cached slot list when the user copies
            // something new.
            let ring = ring::init();
            let _ring_listener_ids = ring::attach(app.handle(), ring);
            ring::controller::start_idle_timer(app.handle().clone());

            // Configure the system tray.
            tray::build(app)?;

            // Hide both windows on startup; they appear via hotkey/tray click.
            for label in ["main", "palette"] {
                if let Some(window) = app.get_webview_window(label) {
                    let _ = window.hide();
                }
            }

            // Phase 6: opt-in local HTTP receiver for browser extension.
            #[cfg(feature = "http_receiver")]
            {
                if settings::is_http_receiver_enabled(&state) {
                    let _ = http_receiver::start(app.handle().clone(), state.clone());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_clips,
            commands::list_clips,
            commands::get_clip,
            commands::toggle_favorite,
            commands::pin_clip,
            commands::delete_clip,
            commands::delete_clips,
            commands::clear_history,
            commands::update_clip_meta,
            commands::list_collections,
            commands::create_collection,
            commands::delete_collection,
            commands::rename_collection,
            commands::assign_to_collection,
            commands::list_snippets,
            commands::search_snippets,
            commands::upsert_snippet,
            commands::delete_snippet,
            commands::copy_snippet_to_clipboard,
            commands::read_image_full,
            commands::read_image_thumb,
            commands::export_db,
            commands::import_db,
            commands::get_settings,
            commands::update_settings,
            commands::show_palette,
            commands::hide_palette,
            commands::show_main,
            commands::hide_main,
            commands::register_hotkey,
            commands::copy_clip_to_clipboard,
            commands::merge_and_paste_clips,
            commands::list_source_apps,
            commands::set_autostart,
            commands::run_backup,
            commands::list_tags,
            // Clipboard Ring
            commands::ring_set_scope,
            commands::ring_reverse,
            commands::ring_forward,
            commands::ring_jump,
            commands::ring_dismiss,
            commands::ring_is_active,
            commands::ring_preview,
            commands::ring_list_sets,
            commands::ring_create_set,
            commands::ring_delete_set,
            commands::ring_add_to_set,
            commands::ring_remove_from_set,
            commands::test_paste,
            commands::transform_clip,
            commands::trigger_screenshot,
            commands::ingest_dropped_files,
            commands::ocr_clip,
            commands::ocr_get,
            commands::list_activity,
            commands::clear_activity,
            // Per-device multi-user
            commands::users_list,
            commands::users_create,
            commands::users_rename,
            commands::users_set_default,
            commands::users_delete,
            commands::users_get_active,
            commands::users_set_active,
        ])
        .on_window_event(|window, event| {
            // Palette hides on blur instead of closing.
            if window.label() == "palette" {
                if let WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
}

/// Entry point. Builds the Tauri application and runs the event loop.
pub fn run() {
    let app = build_app();
    app.build(tauri::generate_context!())
        .expect("error while building Tauri application")
        .run(|_app, _event| {
            // Intentionally a no-op: let the event loop run forever. The tray
            // icon owns the "Quit" action (see `tray::build`), and the
            // single-instance plugin focuses the existing window when a
            // second launch is detected. We never want Tauri to auto-exit
            // just because both windows are hidden on startup.
        });
}
