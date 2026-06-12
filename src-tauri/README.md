# ClipVault backend

Rust + Tauri v2 backend. See `../README.md` and `../docs/architecture.md` for the full picture.

## Quick reference

| Command            | Purpose                                    |
| ------------------ | ------------------------------------------ |
| `cargo build`      | Compile the backend                        |
| `cargo test`       | Run unit + integration tests               |
| `cargo bench`      | Run criterion benchmarks                   |
| `cargo clippy`     | Lints                                      |
| `cargo fmt`        | Format                                     |

## Features

| Feature           | Default | Notes                                              |
| ----------------- | :-----: | -------------------------------------------------- |
| (none)            |   yes   | Core functionality                                 |
| `ocr`             |    -    | Adds Tesseract-based OCR (Phase 6, ~15 MB binary)  |
| `sync`            |    -    | Self-hosted sync client (Phase 6, opt-in)          |
| `http_receiver`   |    -    | Local HTTP receiver for the browser extension      |
| `portable`        |    -    | Stores data next to the executable                 |

Build a portable build:

```bash
cargo build --release --features portable
```

Build with all Phase 6 modules:

```bash
cargo build --release --features ocr,sync,http_receiver
```
