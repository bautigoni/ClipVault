//! Global hotkey handling. The shortcut is registered via the Tauri global-shortcut plugin
//! in `lib::run` and dispatched to the palette window.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::warn;

pub fn register<R: Runtime>(app: &AppHandle<R>, combo: &str) -> Result<(), String> {
    let shortcut = parse(combo).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                // If the ring is currently active, this is the second press of
                // the shared hotkey — cycle the ring instead of toggling the
                // palette. Otherwise show the palette.
                if crate::ring::controller::is_ring_active(app) {
                    crate::ring::controller::cycle_ring(app, /* forward = */ false);
                } else {
                    toggle_palette(app);
                }
            }
        })
        .map_err(|e| e.to_string())
}

pub fn register_ring_forward<R: Runtime>(app: &AppHandle<R>, combo: &str) -> Result<(), String> {
    let shortcut = parse(combo).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                crate::ring::controller::cycle_ring(app, /* forward = */ true);
            }
        })
        .map_err(|e| e.to_string())
}

pub fn register_ring_overlay<R: Runtime>(app: &AppHandle<R>, combo: &str) -> Result<(), String> {
    let shortcut = parse(combo).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                crate::ring::overlay::show_preview(app, &crate::ring::controller::get_ring(app));
            }
        })
        .map_err(|e| e.to_string())
}

pub fn toggle_palette<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("palette") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn parse(combo: &str) -> Result<Shortcut, String> {
    let mut mods = Modifiers::empty();
    let mut key: Option<Code> = None;
    for raw in combo.split('+') {
        let part = raw.trim();
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "commandorcontrol" | "cmd" | "super" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" | "option" => mods |= Modifiers::ALT,
            "meta" | "win" => mods |= Modifiers::META,
            other if !other.is_empty() => {
                key = match other.to_ascii_uppercase().as_str() {
                    "V" => Some(Code::KeyV),
                    "ENTER" | "RETURN" => Some(Code::Enter),
                    "SPACE" => Some(Code::Space),
                    "TAB" => Some(Code::Tab),
                    "ESCAPE" | "ESC" => Some(Code::Escape),
                    s if s.starts_with('F') && s.len() > 1 => {
                        let n: u32 = s[1..].parse().map_err(|_| format!("invalid key '{}'", part))?;
                        match n {
                            1 => Some(Code::F1),
                            2 => Some(Code::F2),
                            3 => Some(Code::F3),
                            4 => Some(Code::F4),
                            5 => Some(Code::F5),
                            6 => Some(Code::F6),
                            7 => Some(Code::F7),
                            8 => Some(Code::F8),
                            9 => Some(Code::F9),
                            10 => Some(Code::F10),
                            11 => Some(Code::F11),
                            12 => Some(Code::F12),
                            _ => return Err(format!("unsupported F-key F{}", n)),
                        }
                    }
                    s if s.len() == 1 => {
                        let c = s.chars().next().unwrap();
                        match c {
                            'A'..='Z' => Some(match c {
                                'A' => Code::KeyA,
                                'B' => Code::KeyB,
                                'C' => Code::KeyC,
                                'D' => Code::KeyD,
                                'E' => Code::KeyE,
                                'F' => Code::KeyF,
                                'G' => Code::KeyG,
                                'H' => Code::KeyH,
                                'I' => Code::KeyI,
                                'J' => Code::KeyJ,
                                'K' => Code::KeyK,
                                'L' => Code::KeyL,
                                'M' => Code::KeyM,
                                'N' => Code::KeyN,
                                'O' => Code::KeyO,
                                'P' => Code::KeyP,
                                'Q' => Code::KeyQ,
                                'R' => Code::KeyR,
                                'S' => Code::KeyS,
                                'T' => Code::KeyT,
                                'U' => Code::KeyU,
                                'V' => Code::KeyV,
                                'W' => Code::KeyW,
                                'X' => Code::KeyX,
                                'Y' => Code::KeyY,
                                'Z' => Code::KeyZ,
                                _ => unreachable!(),
                            }),
                            '0'..='9' => Some(match c {
                                '0' => Code::Digit0,
                                '1' => Code::Digit1,
                                '2' => Code::Digit2,
                                '3' => Code::Digit3,
                                '4' => Code::Digit4,
                                '5' => Code::Digit5,
                                '6' => Code::Digit6,
                                '7' => Code::Digit7,
                                '8' => Code::Digit8,
                                '9' => Code::Digit9,
                                _ => unreachable!(),
                            }),
                            _ => return Err(format!("unsupported key '{}'", part)),
                        }
                    }
                    _ => return Err(format!("unsupported key '{}'", part)),
                }
            }
            _ => {}
        }
    }
    let code = key.ok_or_else(|| "no key in combo".to_string())?;
    Ok(Shortcut::new(Some(mods), code))
}

#[allow(dead_code)]
pub fn default_combo() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV)
}

pub fn warn_if_invalid(combo: &str) {
    if let Err(e) = parse(combo) {
        warn!(combo, error = %e, "invalid hotkey combo");
    }
}
