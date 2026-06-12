//! Auto-paste: simulate a Ctrl+V keypress so the active window receives the
//! clipboard contents. Implemented with `SendInput` on Windows; on other
//! platforms it's a no-op (the feature is Windows-only).
//!
//! Two important Windows-specific gotchas we work around here:
//!
//! 1. `SendInput` from a non-foreground thread is silently dropped by many
//!    apps (browsers, Electron, modern text fields) even when the *target*
//!    window is foreground. The fix is to call `AllowSetForegroundWindow`
//!    with the target pid and bring the target to the foreground before
//!    sending the keystrokes.
//! 2. The 60ms delay we previously used wasn't enough for the palette's
//!    hide animation to finish and the OS to restore focus to the previous
//!    app. 150ms is a more reliable lower bound on real hardware.

/// Write a line to the debug log file directly. We don't go through
/// `tracing` here because we want absolute confidence that the entry
/// lands on disk even if the subscriber isn't wired up correctly.
fn debug_log(msg: &str) {
    use std::io::Write;
    if let Some(dir) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) {
        let path = dir.join("com.clipvault.app").join("debug.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(
                f,
                "[{}] paste: {}",
                chrono::Utc::now().to_rfc3339(),
                msg
            );
        }
    }
}

#[cfg(windows)]
pub fn send_ctrl_v() {
    use std::thread::sleep;
    use std::time::Duration;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
        VK_CONTROL, VK_V,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    debug_log("send_ctrl_v() called");

    fn vk(v: VIRTUAL_KEY) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: v,
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn vk_up(v: VIRTUAL_KEY) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: v,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    unsafe {
        // Let the foreground window recover focus after the palette hides.
        sleep(Duration::from_millis(150));
        debug_log("after 150ms sleep");

        let hwnd: HWND = GetForegroundWindow();
        let mut title_buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize])
        } else {
            String::from("<none>")
        };
        debug_log(&format!("foreground window: '{}'", title));

        // Allow ourselves to set the foreground if needed and ensure the
        // target window is in the foreground. AttachThreadInput lets us
        // share the input state of the foreground thread so the keystrokes
        // are accepted by the target's message pump.
        let mut target_pid = 0u32;
        let target_thread_id = GetWindowThreadProcessId(hwnd, Some(&mut target_pid));
        debug_log(&format!(
            "target pid: {}, target thread id: {}",
            target_pid, target_thread_id
        ));
        if target_pid != 0 {
            let _ = AllowSetForegroundWindow(target_pid);
        }
        let _ = AttachThreadInput(GetCurrentThreadId(), target_thread_id, true);

        let input_size = std::mem::size_of::<INPUT>() as i32;

        // Ctrl down
        let sent = SendInput(&[vk(VK_CONTROL)], input_size);
        debug_log(&format!("SendInput(VK_CONTROL down) = {}", sent));
        if sent != 1 {
            let err = std::io::Error::last_os_error();
            debug_log(&format!("  last error: {} ({})", err, err.raw_os_error().unwrap_or(0)));
        }
        sleep(Duration::from_millis(40));
        // V down
        let sent = SendInput(&[vk(VK_V)], input_size);
        debug_log(&format!("SendInput(VK_V down) = {}", sent));
        if sent != 1 {
            let err = std::io::Error::last_os_error();
            debug_log(&format!("  last error: {} ({})", err, err.raw_os_error().unwrap_or(0)));
        }
        sleep(Duration::from_millis(40));
        // V up
        let sent = SendInput(&[vk_up(VK_V)], input_size);
        debug_log(&format!("SendInput(VK_V up) = {}", sent));
        if sent != 1 {
            let err = std::io::Error::last_os_error();
            debug_log(&format!("  last error: {} ({})", err, err.raw_os_error().unwrap_or(0)));
        }
        sleep(Duration::from_millis(40));
        // Ctrl up
        let sent = SendInput(&[vk_up(VK_CONTROL)], input_size);
        debug_log(&format!("SendInput(VK_CONTROL up) = {}", sent));
        if sent != 1 {
            let err = std::io::Error::last_os_error();
            debug_log(&format!("  last error: {} ({})", err, err.raw_os_error().unwrap_or(0)));
        }

        // Detach our thread from the target's input state.
        let _ = AttachThreadInput(GetCurrentThreadId(), target_thread_id, false);
        debug_log("send_ctrl_v() completed");
    }
}

#[cfg(not(windows))]
pub fn send_ctrl_v() {
    // Auto-paste is Windows-only; on other platforms do nothing so the rest of
    // the app keeps working as a clipboard manager.
}
