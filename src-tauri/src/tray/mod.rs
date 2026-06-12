//! System tray with adaptive (light/dark) icon and menu items.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};
use tracing::info;

pub fn build<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let handle = app.handle();
    let show_main = MenuItem::with_id(handle, "show_main", "Open ClipVault", true, None::<&str>)?;
    let show_palette = MenuItem::with_id(
        handle,
        "show_palette",
        "Quick Paste (Ctrl+Shift+V)",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(handle, "settings", "Settings...", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(handle)?;
    let quit = MenuItem::with_id(handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[&show_main, &show_palette, &sep, &settings, &sep, &quit])?;

    let _tray = TrayIconBuilder::with_id("clipvault-tray")
        .icon(tray_icon())
        .tooltip("ClipVault")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show_main" => toggle_window(app, "main"),
            "show_palette" => toggle_window(app, "palette"),
            "settings" => {
                toggle_window(app, "main");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("window.location.hash = '#/settings'");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle(), "main");
            }
        })
        .build(handle)?;
    info!("system tray initialized");
    Ok(())
}

fn toggle_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn icon_for_taskbar() -> &'static [u8] {
    #[cfg(windows)]
    {
        if is_light_taskbar() {
            include_bytes!("../../icons/tray-light.png")
        } else {
            include_bytes!("../../icons/tray-dark.png")
        }
    }
    #[cfg(not(windows))]
    {
        include_bytes!("../../icons/tray-dark.png")
    }
}

fn tray_icon() -> tauri::image::Image<'static> {
    let bytes = icon_for_taskbar();
    tauri::image::Image::from_bytes(bytes).expect("tray icon bytes are valid")
}

#[cfg(windows)]
fn is_light_taskbar() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize") {
        if let Ok(val) = key.get_value::<u32, _>("SystemUsesLightTheme") {
            return val == 1;
        }
    }
    false
}
