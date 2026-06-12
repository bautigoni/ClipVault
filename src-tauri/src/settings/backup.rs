//! Scheduled + manual backups. The data dir is mirrored into the configured backup dir
//! (default: `<data>/backups/`). Rotated to keep the last 5 archives.

use std::path::Path;

use anyhow::Context;
use tracing::info;

use crate::db::DbPool;
use crate::state::AppState;

pub fn run_backup(state: &AppState) -> anyhow::Result<()> {
    let settings = crate::settings::load_settings(&state.app);
    let dest_root = settings
        .backup_dir
        .clone()
        .unwrap_or_else(|| state.data_dir.join("backups"));
    std::fs::create_dir_all(&dest_root).context("create backup dir")?;

    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string();
    let archive_path = dest_root.join(format!("clipvault-backup-{}.zip", stamp));
    write_archive(&state.db, &state.data_dir, &archive_path)?;
    rotate(&dest_root, 5)?;
    info!(?archive_path, "backup written");
    Ok(())
}

fn write_archive(_db: &DbPool, data_dir: &Path, archive_path: &Path) -> anyhow::Result<()> {
    use std::fs::File;
    use zip::write::FileOptions;

    let file = File::create(archive_path).context("create archive file")?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let db_path = data_dir.join("clipvault.db");
    if db_path.exists() {
        zip.start_file("clipvault.db", options)?;
        let mut f = File::open(&db_path)?;
        std::io::copy(&mut f, &mut zip)?;
    }
    let images_dir = data_dir.join("images");
    if images_dir.exists() {
        append_dir(&mut zip, &images_dir, "images", options)?;
    }
    let thumbs_dir = data_dir.join("thumbs");
    if thumbs_dir.exists() {
        append_dir(&mut zip, &thumbs_dir, "thumbs", options)?;
    }
    zip.finish()?;
    Ok(())
}

fn append_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    src: &Path,
    prefix: &str,
    options: zip::write::FileOptions,
) -> anyhow::Result<()> {
    use std::fs::File;
    for entry in walkdir(src)? {
        let path = entry;
        let rel = path.strip_prefix(src).unwrap_or_else(|_| &path);
        let arcname = format!(
            "{}/{}",
            prefix.trim_end_matches('/'),
            rel.to_string_lossy().replace('\\', "/")
        );
        if path.is_dir() {
            zip.add_directory(arcname, options)?;
        } else {
            zip.start_file(arcname, options)?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

fn walkdir(dir: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        if p.is_dir() {
            for entry in std::fs::read_dir(&p)? {
                let entry = entry?;
                stack.push(entry.path());
            }
            out.push(p);
        } else {
            out.push(p);
        }
    }
    Ok(out)
}

fn rotate(dir: &Path, keep: usize) -> anyhow::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("clipvault-backup-"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    while entries.len() > keep {
        if let Some(old) = entries.first() {
            let _ = std::fs::remove_file(old.path());
        }
        entries.remove(0);
    }
    Ok(())
}
