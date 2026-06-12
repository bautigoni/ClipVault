//! Source application tracking.
//!
//! On Windows we read the foreground window's owning process name and window title via
//! the Win32 API. On non-Windows platforms we return a sensible default so the rest of
//! the pipeline compiles and runs for development/testing.

#[cfg(windows)]
pub fn current_source() -> (Option<String>, Option<String>) {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return (None, None);
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = if len > 0 {
            let s = OsString::from_wide(&title_buf[..len as usize]);
            s.to_string_lossy().to_string()
        } else {
            String::new()
        };

        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            Err(_) => return (Some("unknown".into()), if title.is_empty() { None } else { Some(title) }),
        };
        let mut exe_buf = [0u16; 1024];
        let mut size = exe_buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process_handle, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(exe_buf.as_mut_ptr()), &mut size);
        let _ = windows::Win32::Foundation::CloseHandle(process_handle);
        let exe = if ok.is_ok() && size > 0 {
            let s = OsString::from_wide(&exe_buf[..size as usize]);
            let path = s.to_string_lossy().to_string();
            std::path::Path::new(&path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or(path)
        } else {
            "unknown.exe".to_string()
        };
        let title = if title.is_empty() { None } else { Some(title) };
        (Some(exe), title)
    }
}

#[cfg(not(windows))]
pub fn current_source() -> (Option<String>, Option<String>) {
    (None, None)
}
