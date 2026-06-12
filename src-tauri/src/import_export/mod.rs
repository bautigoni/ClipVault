//! .clipvault archive import/export. Archive = zip with manifest.json + clipvault.db + images/.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportPolicy {
    Skip,
    Overwrite,
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportReport {
    pub clips_added: usize,
    pub clips_skipped: usize,
    pub collections_added: usize,
    pub snippets_added: usize,
    pub errors: Vec<String>,
}

pub fn export_to_zip(archive_path: &Path, db_path: &Path, images_dir: Option<&Path>, thumbs_dir: Option<&Path>) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::Write;
    use zip::write::FileOptions;

    let file = File::create(archive_path).context("create archive")?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest = serde_json::json!({
        "format": "clipvault.v1",
        "created_at": chrono::Utc::now().timestamp(),
        "version": env!("CARGO_PKG_VERSION"),
    });
    zip.start_file("manifest.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;

    if db_path.exists() {
        zip.start_file("clipvault.db", options)?;
        let mut f = File::open(db_path)?;
        std::io::copy(&mut f, &mut zip)?;
    }
    if let Some(dir) = images_dir {
        if dir.exists() {
            append_dir(&mut zip, dir, "images", options)?;
        }
    }
    if let Some(dir) = thumbs_dir {
        if dir.exists() {
            append_dir(&mut zip, dir, "thumbs", options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn import_from_zip(archive_path: &Path, dest_dir: &Path, _policy: ImportPolicy) -> anyhow::Result<ImportReport> {
    use std::fs::File;
    use std::io::Read;

    let file = File::open(archive_path).context("open archive")?;
    let mut zip = ZipArchive::new(file).context("read archive")?;
    std::fs::create_dir_all(dest_dir)?;
    let _ = dest_dir; // dest is just for sanity; we merge into the live DB via callers.
    let mut report = ImportReport::default();
    let counter = AtomicUsize::new(0);
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).context("read entry")?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if name == "manifest.json" {
            let mut s = String::new();
            entry.read_to_string(&mut s).ok();
            // Validate format
            if !s.contains("\"format\": \"clipvault.v1\"") {
                report.errors.push(format!("unknown archive format: {}", s));
            }
            continue;
        }
        if name == "clipvault.db" {
            let tmp = dest_dir.join("imported.db");
            std::fs::create_dir_all(dest_dir)?;
            let mut out = File::create(&tmp)?;
            std::io::copy(&mut entry, &mut out)?;
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            continue;
        }
        if let Some(rest) = name.strip_prefix("images/") {
            let path: PathBuf = dest_dir.join("images").join(rest);
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut out = File::create(&path)?;
            std::io::copy(&mut entry, &mut out)?;
            continue;
        }
        if let Some(rest) = name.strip_prefix("thumbs/") {
            let path: PathBuf = dest_dir.join("thumbs").join(rest);
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut out = File::create(&path)?;
            std::io::copy(&mut entry, &mut out)?;
            continue;
        }
    }
    report.clips_added = counter.load(std::sync::atomic::Ordering::SeqCst);
    Ok(report)
}

fn append_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    src: &Path,
    prefix: &str,
    options: zip::write::FileOptions,
) -> anyhow::Result<()> {
    use std::fs::File;
    for entry in walk(src)? {
        let path = entry;
        let rel = path.strip_prefix(src).unwrap_or_else(|_| &path);
        let arcname = format!("{}/{}", prefix.trim_end_matches('/'), rel.to_string_lossy().replace('\\', "/"));
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

fn walk(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
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
