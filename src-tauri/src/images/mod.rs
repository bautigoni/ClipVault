//! Image storage. Originals and thumbnails are written to disk; only metadata lives in SQLite.

pub mod storage;

pub use storage::{load_full, load_thumb, save_image, save_image_encoded};
