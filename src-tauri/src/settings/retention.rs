//! Background retention sweeper. Runs every 10 minutes; deletes old non-favorite clips.

use std::time::Duration;

use tauri::AppHandle;
use tracing::{debug, info, warn};

use crate::db::repo;
use crate::state::AppState;

pub fn start_sweeper(_app: AppHandle, state: AppState) {
    std::thread::Builder::new()
        .name("clipvault-retention-sweeper".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(10 * 60));
            if let Err(e) = run_once(&state) {
                warn!(?e, "retention sweeper iteration failed");
            }
        })
        .expect("failed to spawn retention sweeper");
}

pub fn run_once(state: &AppState) -> anyhow::Result<()> {
    let settings = crate::settings::load_settings(&state.app);
    if settings.retention_days <= 0 {
        debug!("retention disabled (infinite history)");
    } else {
        let conn = state.db.get()?;
        let cutoff = repo::now_ms() - (settings.retention_days * 24 * 60 * 60 * 1000);
        // The `clips_ad` trigger in the initial migration keeps clips_fts in
        // sync, so a plain DELETE is sufficient.
        let count = conn.execute(
            "DELETE FROM clips WHERE is_favorite = 0 AND is_pinned = 0 AND created_at < ?",
            [cutoff],
        )?;
        if count > 0 {
            info!(count, "retention sweeper pruned clips");
        }
    }
    let pruned = repo::enforce_max_clips(&state.db.get()?, settings.max_clips)?;
    if pruned > 0 {
        info!(pruned, "max-clips cap enforced");
    }
    if settings.backup_enabled {
        if let Err(e) = crate::settings::backup::run_backup(state) {
            warn!(?e, "scheduled backup failed");
        }
    }
    Ok(())
}
