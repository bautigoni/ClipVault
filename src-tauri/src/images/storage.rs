//! Image file management. Originals are written under `<data>/images/<yyyy>/<mm>/<ulid>.png`,
//! thumbnails under `<data>/thumbs/<ulid>.jpg`.
//!
//! The input from the clipboard watcher (`arboard::ImageData`) is **raw RGBA
//! bytes** — not a PNG, not a JPEG. The format detection APIs in the `image`
//! crate can't sniff it because there's no magic header. We therefore build
//! a real `RgbaImage` from the bytes + dimensions and re-encode to PNG so
//! the on-disk format is self-describing and re-readable by anything.

use std::path::Path;

use anyhow::Context;
use chrono::Datelike;
use image::imageops::FilterType;
use image::{ImageBuffer, RgbaImage};

pub const THUMB_SIZE: u32 = 256;

/// Save an image clip from raw RGBA pixels.
///
/// `rgba_bytes.len()` must equal `width * height * 4`; otherwise we return an
/// error rather than panic, so a malformed clipboard payload doesn't crash
/// the watcher thread.
pub fn save_image(
    images_dir: &Path,
    id: &str,
    width: u32,
    height: u32,
    rgba_bytes: &[u8],
) -> anyhow::Result<(String, String)> {
    anyhow::ensure!(
        rgba_bytes.len() == (width as usize) * (height as usize) * 4,
        "rgba_bytes length {} doesn't match width*height*4 = {} ({}x{})",
        rgba_bytes.len(),
        (width as usize) * (height as usize) * 4,
        width,
        height,
    );

    // Build an `RgbaImage` from the raw pixel buffer. This is the canonical
    // way to materialize RGBA data in the `image` crate; from here we can
    // re-encode to whatever container format we want.
    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba_bytes.to_vec())
        .ok_or_else(|| anyhow::anyhow!("failed to build RgbaImage from raw buffer"))?;

    let now = chrono::Utc::now();
    let subdir = images_dir
        .join(format!("{:04}", now.year()))
        .join(format!("{:02}", now.month()));
    std::fs::create_dir_all(&subdir).with_context(|| format!("create image dir {:?}", subdir))?;
    let original_path = subdir.join(format!("{}.png", id));
    img.save_with_format(&original_path, image::ImageFormat::Png)
        .with_context(|| format!("write original png {:?}", original_path))?;
    let rel_original = relative_path(images_dir, &original_path);

    // Thumbnail: resize to fit within THUMB_SIZE on the longest edge, then
    // store as JPEG (much smaller than PNG for photos). `RgbaImage::resize`
    // isn't a method in image 0.25, so we use the free function in
    // `image::imageops` which works on any `GenericImageView`.
    let thumb_dir = images_dir
        .parent()
        .map(|p| p.join("thumbs"))
        .unwrap_or_else(|| images_dir.join("thumbs"));
    std::fs::create_dir_all(&thumb_dir)?;
    let thumb_path = thumb_dir.join(format!("{}.jpg", id));
    let thumb = image::imageops::resize(&img, THUMB_SIZE, THUMB_SIZE, FilterType::Triangle);
    // `imageops::resize` returns an `RgbaImage`; convert to RGB8 (dropping the
    // alpha channel) before saving as JPEG, which doesn't support alpha.
    let rgb: image::RgbImage = image::ImageBuffer::from_fn(thumb.width(), thumb.height(), |x, y| {
        let p = thumb.get_pixel(x, y);
        image::Rgb([p[0], p[1], p[2]])
    });
    rgb.save_with_format(&thumb_path, image::ImageFormat::Jpeg)
        .with_context(|| format!("write thumb jpeg {:?}", thumb_path))?;
    // Thumb path is computed relative to `images_dir`'s parent (the data dir),
    // since `images_dir` and `thumb_dir` are siblings, not nested.
    let data_dir = images_dir.parent().unwrap_or(images_dir);
    let rel_thumb = relative_path(data_dir, &thumb_path);
    Ok((rel_original, rel_thumb))
}

/// Save an image clip from a pre-encoded image format (PNG, JPEG, etc).
/// Used when the caller already has a real image file from the clipboard
/// (e.g. drag-and-drop or file-system import) rather than raw RGBA pixels.
#[allow(unused_variables)]
pub fn save_image_encoded(
    images_dir: &Path,
    id: &str,
    encoded: &[u8],
    format: image::ImageFormat,
) -> anyhow::Result<(String, String)> {
    let img = image::load_from_memory(encoded)
        .context("decode encoded image for thumbnail")?;
    let (w, h) = (img.width(), img.height());
    let rgba8 = img.to_rgba8();
    // Recurse through save_image with the freshly decoded RGBA so the on-disk
    // format is always PNG (uniform across ingest paths).
    let mut pixels = Vec::with_capacity(rgba8.len());
    pixels.extend_from_slice(&rgba8);
    save_image(images_dir, id, w, h, &pixels)
}

fn relative_path(base: &Path, target: &Path) -> String {
    target
        .strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| target.to_string_lossy().to_string())
}

pub fn load_full(data_dir: &Path, rel_path: &str) -> anyhow::Result<Vec<u8>> {
    // `rel_path` is stored as `<yyyy>/<mm>/<id>.png` (relative to `images_dir`,
    // which is `<data>/images`). We therefore join under `data_dir/images/`.
    let full = data_dir.join("images").join(rel_path);
    Ok(std::fs::read(&full).with_context(|| format!("read image {:?}", full))?)
}

pub fn load_thumb(data_dir: &Path, rel_path: &str) -> anyhow::Result<Vec<u8>> {
    // `rel_path` is stored relative to `data_dir` (see `save_image`), so join
    // it directly under `data_dir`. The thumb file lives under `data_dir/thumbs/`.
    let full = data_dir.join(rel_path);
    Ok(std::fs::read(&full).with_context(|| format!("read thumb {:?}", full))?)
}
