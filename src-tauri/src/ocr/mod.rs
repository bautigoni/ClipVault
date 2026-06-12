//! Optional OCR module (feature `ocr`). Stubs out unless the feature is enabled.

#[cfg(feature = "ocr")]
pub fn run_ocr(_png_bytes: &[u8]) -> anyhow::Result<String> {
    // Implementation note: integrate `tesseract-rs` here. The default crate adds ~15 MB to
    // the binary, which is why OCR is feature-gated.
    Ok(String::new())
}

#[cfg(not(feature = "ocr"))]
pub fn run_ocr(_png_bytes: &[u8]) -> anyhow::Result<String> {
    Ok(String::new())
}
