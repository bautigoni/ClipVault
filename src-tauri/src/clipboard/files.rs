//! Windows file-drop list (CF_HDROP) reading + writing. On non-Windows platforms this is a no-op.

/// Write a list of file paths to the Windows clipboard as a CF_HDROP.
#[cfg(windows)]
pub fn write_file_list(paths: &[String]) -> anyhow::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{CloseClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    if paths.is_empty() {
        return Ok(());
    }
    unsafe {
        if OpenClipboard(None).is_err() {
            return Err(anyhow::anyhow!("OpenClipboard failed"));
        }
        // DROPFILES struct: DWORD (files) + POINT (zero) + BOOL (wide) = 20 bytes
        let header_size = std::mem::size_of::<u32>() * 2 + std::mem::size_of::<isize>() * 2;
        let mut payload: Vec<u16> = Vec::new();
        for path in paths {
            for c in std::ffi::OsStr::new(path).encode_wide() {
                payload.push(c);
            }
            payload.push(0);
        }
        payload.push(0);
        let total_size = header_size + payload.len() * 2;
        let hg = GlobalAlloc(GMEM_MOVEABLE, total_size).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let ptr = GlobalLock(hg);
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err(anyhow::anyhow!("GlobalLock returned null"));
        }
        // Files count
        std::ptr::write_unaligned::<u32>(ptr as *mut u32, paths.len() as u32);
        // POINT (0,0) - skip
        // BOOL wide = TRUE
        std::ptr::write_unaligned::<u32>((ptr as *mut u8).add(8) as *mut u32, 1);
        // File list offset
        std::ptr::write_unaligned::<u32>((ptr as *mut u8).add(20) as *mut u32, header_size as u32);
        // Copy wide strings
        let dst = (ptr as *mut u8).add(header_size);
        std::ptr::copy_nonoverlapping(payload.as_ptr() as *const u8, dst, payload.len() * 2);
        let _ = GlobalUnlock(hg);
        let result = SetClipboardData(15, HANDLE(hg.0 as _));
        let _ = CloseClipboard();
        if result.is_err() {
            return Err(anyhow::anyhow!("SetClipboardData failed"));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn write_file_list(_paths: &[String]) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(windows)]
pub fn read_file_list() -> Option<Vec<String>> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, OpenClipboard,
    };
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

    const CF_HDROP: u32 = 15;

    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let result = (|| -> Option<Vec<String>> {
            let handle = GetClipboardData(CF_HDROP).ok()?;
            if handle.0.is_null() {
                return None;
            }
            let hg = HGLOBAL(handle.0 as *mut _);
            let ptr = GlobalLock(hg);
            if ptr.is_null() {
                return None;
            }
            let count = std::ptr::read_unaligned::<u32>(ptr as *const u32);
            if count == 0 {
                let _ = GlobalUnlock(hg);
                return Some(vec![]);
            }
            let array_offset = std::mem::size_of::<u32>() as isize;
            let mut paths = Vec::with_capacity(count as usize);
            for i in 0..count {
                let entry_ptr = (ptr as *const u8).offset(array_offset + (i as isize) * std::mem::size_of::<isize>() as isize);
                let wide_ptr = std::ptr::read_unaligned::<*const u16>(entry_ptr as *const *const u16);
                if wide_ptr.is_null() {
                    continue;
                }
                let mut len = 0usize;
                while std::ptr::read_unaligned::<u16>(wide_ptr.add(len)) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(wide_ptr, len);
                let os = OsString::from_wide(slice);
                paths.push(os.to_string_lossy().to_string());
            }
            let _ = GlobalUnlock(hg);
            Some(paths)
        })();
        let _ = CloseClipboard();
        result
    }
}

#[cfg(not(windows))]
pub fn read_file_list() -> Option<Vec<String>> {
    None
}
