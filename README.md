# ClipVault

> Privacy-first, local-only Windows clipboard manager built with Tauri v2 + Rust + React + SQLite FTS5.

**👉 Want to try it? Grab the installer:** [**Latest release**](https://github.com/bautigoni/ClipVault/releases/latest) · [**Landing page**](https://bautigoni.github.io/ClipVault/) · [**Website source**](./landing)

ClipVault captures everything you copy - text, URLs, images, and files - and lets you search, organize, and paste it back in a flash. There is no cloud, no telemetry, no account. Your clipboard history lives in a single SQLite database on your machine, indexed with full-text search that returns results in under 50 ms even at 100k+ entries.

## Quick install (Windows 10/11)

1. Download `ClipVault_0.1.0_x64-setup.exe` from the [**latest release**](https://github.com/bautigoni/ClipVault/releases/latest).
2. Double-click the installer. The NSIS wizard handles the rest. Per-user install is supported (no admin needed).
3. Launch ClipVault from the Start menu. It lives in the system tray.
4. Hit <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> anywhere to open the Quick Paste palette. Type a few letters, hit <kbd>Enter</kbd>. Done.

That's it. Your clipboard history is stored locally in `%APPDATA%\com.clipvault.app\`. Delete that folder any time to wipe everything.

### Build from source

Requires [Rust](https://rustup.rs/) (stable) and [Node](https://nodejs.org/) 20+.

```bash
git clone https://github.com/bautigoni/ClipVault
cd ClipVault
npm install
npm run tauri:build
# installer lands in src-tauri/target/release/bundle/
```

First build takes ~10 minutes. Subsequent incremental builds are ~2 minutes.

### Troubleshooting install

- **"Another app is using Ctrl+Shift+V"**: open ClipVault's Settings and rebind the hotkey.
- **Win+V conflict**: Windows 10/11 has a built-in clipboard history. Turn it off in **Settings → System → Clipboard → Clipboard history (off)** if you want ClipVault to use that combo.
- **Antivirus flag**: ClipVault is unsigned open-source. The first run may trigger a SmartScreen prompt — click *More info → Run anyway*. See [issue tracker](https://github.com/bautigoni/ClipVault/issues) for signing status.

## Highlights

- **Instant search** - SQLite FTS5 with BM25 ranking + fuzzy fallback for typo tolerance.
- **Global hotkey** - `Ctrl+Shift+V` opens the Quick Paste palette from anywhere.
- **Timeline, favorites, collections, tags** - organize history however you like.
- **Images and files** - thumbnails stored on disk; originals never held in memory.
- **Snippets** - dedicated area for SQL queries, bash scripts, and code with syntax highlighting.
- **System tray** - adaptive light/dark icon; one-click access to everything.
- **Portable mode** - run from a USB drive with data stored next to the executable.
- **Import / export** - single `.clipvault` ZIP archive of your entire history.
- **Backups** - scheduled, with rotation.
- **Installer** - NSIS + MSI, with code-signing placeholder.

## Architecture (one-paragraph)

A Rust backend (Tauri v2) runs three background threads - a clipboard watcher that polls every 500 ms, a retention sweeper that prunes old clips every 10 minutes, and a backup scheduler. Clip data lands in a WAL-mode SQLite database with an FTS5 virtual table mirroring `clips.text_preview`, `source_app`, and `source_title`. All commands are async Tauri handlers backed by an `r2d2` connection pool. The React + TypeScript frontend renders two windows - a full `main` window (timeline, favorites, collections, snippets, settings, images) and a frameless always-on-top `palette` window. State is managed with Zustand, server state with TanStack Query.

For the full picture see [`docs/architecture.md`](docs/architecture.md).

## Testing

### Frontend (Vitest)

```bash
npm test          # one-shot run
npm run test:watch
```

Coverage focuses on:

- `src/lib/utils.ts` – formatting, debounce, date grouping
- `src/lib/schemas.ts` – zod validation for forms (settings, collections, snippets, hotkey)
- `src/features/timeline/grouping.test.ts` – the `buildTimelineRows` pure function

### Backend (Rust)

```bash
cd src-tauri
cargo test                          # unit + integration tests
cargo bench --bench search          # criterion search latency bench
```

Highlights:

- `tests/db.rs` covers migrations, FTS5 search, dedup, retention, suppress flag, and the
  `blake3` content-hash helper.

## Quick start (development)

> **Prerequisites**: Rust stable (>= 1.77.2), Node.js 20+, pnpm 9+, Microsoft Visual Studio Build Tools (Desktop development with C++) for Tauri on Windows, WebView2 runtime.

```bash
pnpm install
pnpm tauri:dev
```

This launches Vite + the Tauri shell. The first run will take a while to compile the Rust dependencies.

## Production build

```bash
pnpm tauri:build
```

Produces:

- `src-tauri/target/release/clipvault.exe` - the binary.
- `src-tauri/target/release/bundle/nsis/clipvault-setup.exe` - NSIS installer.
- `src-tauri/target/release/bundle/msi/ClipVault_0.1.0_x64_en-US.msi` - MSI.

## Portable build

```bash
resources\build-portable.cmd
```

Produces `target/portable/clipvault-portable/` - a folder you can copy to a USB drive.

## Privacy

- No cloud. No telemetry. No analytics. No account.
- Network calls only happen if you opt in to the self-hosted sync or the browser extension receiver; both are off by default.
- All data lives at `%APPDATA%\com.clipvault.app\` (or next to the binary in portable mode).

## Keyboard shortcuts

| Context        | Key                     | Action                |
| -------------- | ----------------------- | --------------------- |
| Anywhere       | `Ctrl+Shift+V`          | Open Quick Paste      |
| Palette        | `↑` / `↓`               | Move selection        |
| Palette        | `Enter`                 | Copy + paste          |
| Palette        | `Esc`                   | Close palette         |
| Timeline       | `Ctrl+K`                | Focus search          |
| Settings       | (click `Save`)          | Persist changes       |

See [`docs/keyboard.md`](docs/keyboard.md) for the full keymap.

## Project layout

```
ClipVault/
├── src/                    React + TypeScript frontend
│   ├── routes/             Pages (Timeline, Favorites, Collections, Snippets, ...)
│   ├── components/         AppShell, CommandPalette, ClipRow, ...
│   ├── stores/             Zustand stores
│   ├── lib/                Tauri wrappers, utils
│   └── styles/             Tailwind entry
├── src-tauri/              Rust backend
│   └── src/
│       ├── clipboard/      Watcher, source capture, file-drop list
│       ├── db/             SQLite pool, migrations, repository
│       ├── search/         FTS5 + fuzzy fallback
│       ├── images/         Storage + thumbnails
│       ├── settings/       Persistence, retention sweeper, backup
│       ├── snippets/       Language defaults
│       ├── tray/           System tray
│       ├── hotkey/         Global shortcut parsing/registration
│       ├── import_export/  .clipvault ZIP
│       └── commands.rs     Tauri command surface
├── installer/              NSIS + WiX templates
├── resources/              Build scripts, icons
├── clipvault-extension/    Browser extension (MV3)
└── docs/                   Architecture, keyboard map
```

## License

[MIT](LICENSE)
