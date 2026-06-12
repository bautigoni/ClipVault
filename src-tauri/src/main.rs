// ClipVault entry point. All real logic lives in the lib crate so it can be reused by tests/benches.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Log panics AND tracing events to a file so release-mode crashes (no
    // console) and the auto-paste flow are diagnosable from disk.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(dir) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) {
            let log_dir = dir.join("com.clipvault.app");
            let _ = std::fs::create_dir_all(&log_dir);
            let log_path = log_dir.join("panic.log");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                use std::io::Write;
                let _ = writeln!(f, "---- panic at {} ----", chrono::Utc::now().to_rfc3339());
                let _ = writeln!(f, "{}", info);
            }
        }
        default_hook(info);
    }));

    // Mirror tracing output to %APPDATA%\com.clipvault.app\debug.log so we
    // can see what the auto-paste / watcher / hotkey paths are doing
    // without a console attached.
    let log_path = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .map(|d| d.join("com.clipvault.app").join("debug.log"));
    if let Some(p) = log_path {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Best-effort: append a session-start marker. Real tracing output
        // is wired up in `clipvault_lib::run` via a custom layer if the
        // env var CLIPVAULT_LOG_FILE is set; this is the cheap fallback so
        // the file always has *something* to grep.
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
        {
            use std::io::Write;
            let _ = writeln!(
                f,
                "---- session start at {} ----",
                chrono::Utc::now().to_rfc3339()
            );
        }
        // Tell the lib where to write the log. The lib will set up a
        // tracing-subscriber file layer that reads this env var.
        unsafe {
            std::env::set_var("CLIPVAULT_LOG_FILE", &p);
        }
    }

    clipvault_lib::run();
}
