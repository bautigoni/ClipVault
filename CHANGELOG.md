# Changelog

All notable changes to ClipVault are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Initial Tauri v2 + Rust + React + TypeScript monorepo.
- SQLite + FTS5 storage with WAL, NORMAL sync, mmap, and keyset pagination.
- Migration runner with the `clips`, `clips_fts`, `collections`, `tags`, `snippets`,
  `snippets_fts`, `images`, `file_clips`, `clip_ocr`, and `settings` tables.
- Clipboard watcher that polls text, images, and file-drop lists every 500 ms,
  captures the foreground app on Windows via Win32, dedupes by blake3 hash, and
  honors a 1.5s suppress flag for Quick Paste.
- Search engine combining FTS5 BM25 with a Levenshtein-based fuzzy fallback.
- Command palette (`Ctrl+Shift+V`) with debounced search, arrow-key navigation,
  and instant paste.
- System tray with adaptive light/dark icon, quick paste, and quit menu.
- Timeline grouping (Today / Yesterday / Last Week / Last Month / Last Year / Older).
- Favorites, collections, tags.
- Image and file clip support with on-disk storage and JPEG thumbnails.
- Snippets module with FTS5 search.
- Source-app filter and per-app exclude list.
- Settings UI for autostart, hotkey, retention, max clips, theme, storage dir,
  backup folder, and import/export.
- Retention sweeper (every 10 min) and scheduled backups (last 5 rotated).
- Import / export of `.clipvault` ZIP archives.
- NSIS + MSI installer templates; portable build script.
- Feature-gated OCR, sync, and HTTP-receiver modules for Phase 6.
- Browser extension (MV3) for sending page selections to the local HTTP receiver.

### Security

- CSP locked down in `tauri.conf.json`.
- Capabilities are split between the `main` and `palette` windows; the palette
  has the minimum required to read and write the clipboard.
- Network egress requires explicit opt-in.
