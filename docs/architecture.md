# ClipVault Architecture

This document explains the moving parts of ClipVault and how they fit together.

## Data flow

```mermaid
flowchart LR
  Win["Windows Clipboard (CF_UNICODETEXT / CF_DIB / CF_HDROP)"]
  Watcher["Rust Watcher Thread (500 ms poll)"]
  Source["Source App Capture (GetForegroundWindow)"]
  Hash["blake3 Hash + Dedup"]
  DB[("SQLite + FTS5 (WAL)")]
  ImgDisk[("Image Files + Thumbnails on Disk")]
  FTS["clips_fts (BM25, snippet)"]
  Cmd["Tauri Commands (async, r2d2 pool)"]
  Tray["System Tray (adaptive icon)"]
  Hotkey["Global Hotkey (Ctrl+Shift+V)"]
  Palette["Palette Window (React)"]
  Main["Main Window (React Router)"]
  Settings["Settings Store (tauri-plugin-store)"]
  Backup["Backup / Export (zip)"]
  OCR["OCR (opt-in, Phase 6)"]

  Win --> Watcher --> Source --> Hash --> DB
  Watcher --> ImgDisk
  Watcher --> OCR
  DB <--> FTS
  Hash --> Cmd
  DB --> Cmd
  FTS --> Cmd
  Cmd --> Palette
  Cmd --> Main
  Tray --> Main
  Hotkey --> Palette
  Settings --> Watcher
  Settings --> Hotkey
  DB --> Backup
```

## Database schema

```mermaid
erDiagram
  clips ||--o| images : "1:0..1"
  clips ||--o| file_clips : "1:0..1"
  clips }o--o| collections : "n:0..1"
  clips ||--o{ tags : "1:n"
  clips_fts ||--|| clips : "FTS mirror"
  snippets_fts ||--|| snippets : "FTS mirror"

  clips {
    TEXT id PK
    TEXT type
    TEXT content_hash
    TEXT text_preview
    INTEGER byte_size
    TEXT source_app
    TEXT source_title
    INTEGER is_favorite
    INTEGER is_pinned
    TEXT collection_id FK
    INTEGER created_at
    INTEGER usage_count
  }
  collections { TEXT id PK; TEXT name UK; TEXT icon; INTEGER created_at }
  tags { TEXT clip_id FK; TEXT tag }
  images { TEXT clip_id PK FK; TEXT path; INTEGER width; INTEGER height; TEXT thumb_path; TEXT mime }
  file_clips { TEXT clip_id PK FK; TEXT paths }
  snippets { TEXT id PK; TEXT title; TEXT language; TEXT body; INTEGER is_favorite; INTEGER created_at; INTEGER updated_at }
  settings { TEXT key PK; TEXT value }
```

## Clipboard capture lifecycle

```mermaid
sequenceDiagram
  participant OS as Windows Clipboard
  participant W as Watcher Thread
  participant S as Source Capture
  participant H as Hash/Dedup
  participant DB as SQLite
  participant FE as Frontend

  loop every 500ms
    W->>OS: read_text, read_image, CF_HDROP
    W->>S: GetForegroundWindow + GetWindowThreadProcessId
    S-->>W: exe + window title
    W->>H: blake3(content)
    H->>H: dedup window (60s)
    H->>DB: insert or bump usage_count
    DB-->>W: row id
    W-->>FE: emit("clip://created", row)
  end
```

## Quick Paste flow

```mermaid
sequenceDiagram
  participant User
  participant Palette
  participant Rust
  participant OS as Windows Clipboard
  User->>Palette: Ctrl+Shift+V
  Palette->>Rust: show window + focus input
  User->>Palette: type "server"
  Palette->>Rust: search_clips("server")
  Rust-->>Palette: top 50 FTS results
  User->>Palette: Enter on row
  Palette->>Rust: copy_clip_to_clipboard(id)
  Rust->>OS: SetClipboardData(...)
  Rust->>Rust: arm suppress flag (1.5s)
  Rust-->>Palette: hide window
  Note over Rust,OS: Watcher skips re-recording the same hash
```

## Performance budget

| Metric               | Target          | Where it's enforced                                      |
| -------------------- | --------------- | -------------------------------------------------------- |
| Cold start           | < 1 s           | `tauri-plugin-single-instance`, deferred DB open         |
| Idle RAM             | < 100 MB        | `r2d2` pool (8 conns), virtualized lists, no in-mem cache |
| Search p95           | < 50 ms         | FTS5 + BM25, keyset pagination, prepared statements      |
| Watcher poll         | 500 ms          | `clipboard::POLL_INTERVAL`                               |
| Retention sweep      | every 10 min    | `settings::retention::start_sweeper`                     |

## Threading model

| Thread                          | Owner              | Purpose                                                  |
| ------------------------------- | ------------------ | -------------------------------------------------------- |
| Main (Tauri event loop)         | Tauri runtime      | Commands, window events                                  |
| `clipvault-clipboard-watcher`   | Rust               | Polls the clipboard, writes to SQLite                    |
| `clipvault-retention-sweeper`   | Rust               | Periodically prunes old clips + enforces max-clip cap    |
| Tokio runtime (default)         | Rust               | Powers async Tauri commands                              |
| Renderer (main)                 | WebView2           | Timeline / favorites / settings UI                      |
| Renderer (palette)              | WebView2           | Command palette UI                                       |

## Privacy

- `tauri.conf.json` CSP locks down `connect-src`, `script-src`, and `img-src`.
- Capabilities for each window explicitly grant only what is required.
- The `default-src 'self'` directive prevents external resources.
- Network requests require an opt-in feature flag (`sync` or `http_receiver`).

## Build & distribution

| Target     | How                                                | Output                                       |
| ---------- | -------------------------------------------------- | -------------------------------------------- |
| Dev        | `pnpm tauri:dev`                                   | Hot-reloading dev shell                      |
| Release    | `pnpm tauri:build`                                 | `clipvault.exe` + NSIS + MSI bundles         |
| Portable   | `resources/build-portable.cmd`                     | `target/portable/clipvault-portable/`        |
