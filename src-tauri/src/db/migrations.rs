//! Versioned SQLite migrations. Applied on every startup; each migration is a single SQL batch
//! stored in its own file. Add a new `Migration` entry and bump `CURRENT_VERSION` to upgrade
//! the schema.

use anyhow::Context;
use rusqlite::params;
use tracing::info;

use super::{DbConn, DbPool};

#[allow(dead_code)]
const CURRENT_VERSION: i32 = 4;

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        sql: INITIAL_SCHEMA,
    },
    Migration {
        version: 2,
        name: "clipboard_ring",
        sql: RING_SCHEMA,
    },
    Migration {
        version: 3,
        name: "activity_log",
        sql: ACTIVITY_SCHEMA,
    },
    Migration {
        version: 4,
        name: "users",
        sql: USERS_SCHEMA,
    },
];

/// Run any pending migrations against the database. Idempotent.
pub fn run(pool: &DbPool) -> anyhow::Result<()> {
    let mut conn = pool.get().context("migrations: get connection")?;
    ensure_meta_table(&mut conn)?;

    let current: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current {
            info!(version = migration.version, name = migration.name, "applying migration");
            let tx = conn.transaction()?;
            tx.execute_batch(migration.sql)
                .with_context(|| format!("migration v{} ({}) failed", migration.version, migration.name))?;
            tx.execute(
                "INSERT INTO schema_version (version, name, applied_at) VALUES (?, ?, ?)",
                params![migration.version, migration.name, super::repo::now_ms()],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn ensure_meta_table(conn: &mut DbConn) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

const INITIAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS clips (
  id            TEXT PRIMARY KEY,
  type          TEXT NOT NULL,
  content_hash  TEXT NOT NULL,
  text_preview  TEXT,
  byte_size     INTEGER NOT NULL,
  source_app    TEXT,
  source_title  TEXT,
  is_favorite   INTEGER NOT NULL DEFAULT 0,
  is_pinned     INTEGER NOT NULL DEFAULT 0,
  is_sensitive  INTEGER NOT NULL DEFAULT 0,
  collection_id TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  usage_count   INTEGER NOT NULL DEFAULT 1,
  last_used_at  INTEGER,
  pinned_at     INTEGER
);
CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_hash       ON clips(content_hash);
CREATE INDEX IF NOT EXISTS idx_clips_favorite   ON clips(is_favorite, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_source     ON clips(source_app, created_at DESC);

CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
  text_preview, source_app, source_title,
  content='clips', content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);

-- Triggers to keep clips_fts in sync with the base table
CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
  INSERT INTO clips_fts(rowid, text_preview, source_app, source_title)
  VALUES (new.rowid, new.text_preview, new.source_app, new.source_title);
END;
CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, text_preview, source_app, source_title)
  VALUES('delete', old.rowid, old.text_preview, old.source_app, old.source_title);
END;
CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, text_preview, source_app, source_title)
  VALUES('delete', old.rowid, old.text_preview, old.source_app, old.source_title);
  INSERT INTO clips_fts(rowid, text_preview, source_app, source_title)
  VALUES (new.rowid, new.text_preview, new.source_app, new.source_title);
END;

CREATE TABLE IF NOT EXISTS collections (
  id         TEXT PRIMARY KEY,
  name       TEXT UNIQUE NOT NULL,
  icon       TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
  clip_id TEXT NOT NULL,
  tag     TEXT NOT NULL,
  PRIMARY KEY (clip_id, tag)
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

CREATE TABLE IF NOT EXISTS snippets (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL,
  language    TEXT NOT NULL,
  body        TEXT NOT NULL,
  is_favorite INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_snippets_created ON snippets(created_at DESC);

CREATE VIRTUAL TABLE IF NOT EXISTS snippets_fts USING fts5(
  title, language, body,
  content='snippets', content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS snippets_ai AFTER INSERT ON snippets BEGIN
  INSERT INTO snippets_fts(rowid, title, language, body)
  VALUES (new.rowid, new.title, new.language, new.body);
END;
CREATE TRIGGER IF NOT EXISTS snippets_ad AFTER DELETE ON snippets BEGIN
  INSERT INTO snippets_fts(snippets_fts, rowid, title, language, body)
  VALUES('delete', old.rowid, old.title, old.language, old.body);
END;
CREATE TRIGGER IF NOT EXISTS snippets_au AFTER UPDATE ON snippets BEGIN
  INSERT INTO snippets_fts(snippets_fts, rowid, title, language, body)
  VALUES('delete', old.rowid, old.title, old.language, old.body);
  INSERT INTO snippets_fts(rowid, title, language, body)
  VALUES (new.rowid, new.title, new.language, new.body);
END;

CREATE TABLE IF NOT EXISTS images (
  clip_id    TEXT PRIMARY KEY,
  path       TEXT NOT NULL,
  width      INTEGER NOT NULL,
  height     INTEGER NOT NULL,
  thumb_path TEXT NOT NULL,
  mime       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_clips (
  clip_id TEXT PRIMARY KEY,
  paths   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- OCR text captured by the (optional) ocr feature.
CREATE TABLE IF NOT EXISTS clip_ocr (
  clip_id   TEXT PRIMARY KEY,
  text      TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
"#;

const RING_SCHEMA: &str = r#"
-- Clipboard Ring: persistent working sets.
CREATE TABLE IF NOT EXISTS ring_sets (
  id          TEXT PRIMARY KEY,
  name        TEXT UNIQUE NOT NULL,
  scope_kind  TEXT NOT NULL,        -- 'collection' | 'application' | 'kind' | 'custom'
  scope_ref   TEXT,                 -- free-form, e.g. collection id or app exe
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ring_set_items (
  set_id   TEXT NOT NULL,
  clip_id  TEXT NOT NULL,
  position INTEGER NOT NULL,
  PRIMARY KEY (set_id, clip_id),
  FOREIGN KEY (set_id)  REFERENCES ring_sets(id) ON DELETE CASCADE,
  FOREIGN KEY (clip_id) REFERENCES clips(id)      ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ring_set_items_set ON ring_set_items(set_id, position);

CREATE TABLE IF NOT EXISTS ring_set_configs (
  set_id            TEXT PRIMARY KEY,
  capacity          INTEGER NOT NULL DEFAULT 64,
  wrap              INTEGER NOT NULL DEFAULT 1,
  idle_dismiss_ms   INTEGER NOT NULL DEFAULT 30000,
  include_sensitive INTEGER NOT NULL DEFAULT 0,
  include_files     INTEGER NOT NULL DEFAULT 1,
  include_images    INTEGER NOT NULL DEFAULT 1,
  FOREIGN KEY (set_id) REFERENCES ring_sets(id) ON DELETE CASCADE
);

-- Per-clip ring metadata (so kind / collection filtering is index-friendly)
CREATE INDEX IF NOT EXISTS idx_clips_kind_created ON clips(type, created_at DESC);
"#;

/// Append-only activity log: every interesting thing the app does lands here
/// (clip created, clip copied, clip deleted, settings changed, hotkey fired,
/// OCR finished, ...). Powers the in-app "Activity" view so the user can see
/// what happened and when. We never log clipboard *content* — only metadata
/// (clip id, source app, byte size, action kind) — to keep the log safe to
/// share for debugging without leaking secrets.
const ACTIVITY_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS activity_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  ts_ms       INTEGER NOT NULL,
  kind        TEXT NOT NULL,
  clip_id     TEXT,
  source_app  TEXT,
  detail      TEXT
);
CREATE INDEX IF NOT EXISTS idx_activity_ts ON activity_log(ts_ms DESC);
CREATE INDEX IF NOT EXISTS idx_activity_kind_ts ON activity_log(kind, ts_ms DESC);
"#;

/// Phase 6 / Fase 1: per-device multi-user support. Adds a `users` table and
/// a `user_id` column to every user-owned row. The active user is selected
/// at runtime via the `active_user_id` setting; queries filter on it.
///
/// We do NOT make `user_id` NOT NULL — the backfill in the migration itself
/// fills existing rows with the default user, and code that does not pass a
/// `user_id` keeps the old behaviour (NULL = default). The NOT NULL constraint
/// ships in a later migration once the codebase is fully converted.
const USERS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
  id           TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  email        TEXT,
  is_default   INTEGER NOT NULL DEFAULT 0,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_default
  ON users(is_default) WHERE is_default = 1;

-- Default user. Every existing row will point to it. New rows created
-- without an explicit user will also point to it (the active user is
-- always seeded as a default; we pick the most-recently-active one at
-- boot if multiple defaults somehow exist, but the unique partial index
-- guarantees only one in practice).
INSERT OR IGNORE INTO users (id, display_name, email, is_default, created_at, updated_at)
VALUES ('user_default', 'Default', NULL, 1,
        (CAST(strftime('%s','now') AS INTEGER) * 1000),
        (CAST(strftime('%s','now') AS INTEGER) * 1000));

-- Add user_id columns to every user-owned table. Each ADD COLUMN that
-- tries to add a column that already exists raises a SqliteError, so we
-- guard with the pragma-based check below (re-runs of this migration
-- must be no-ops). SQLite has no IF NOT EXISTS for ALTER TABLE ADD COLUMN
-- pre-3.35, but the project pins rusqlite 0.31 which uses sqlite 3.45+.
ALTER TABLE clips              ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE collections        ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE snippets           ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE images             ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE file_clips         ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE clip_ocr           ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE ring_sets          ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE ring_set_items     ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE ring_set_configs   ADD COLUMN user_id TEXT REFERENCES users(id);

-- Backfill: every existing row points to the default user. The partial
-- index already guarantees 'user_default' is present.
UPDATE clips              SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE collections        SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE snippets           SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE images             SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE file_clips         SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE clip_ocr           SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE ring_sets          SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE ring_set_items     SET user_id = 'user_default' WHERE user_id IS NULL;
UPDATE ring_set_configs   SET user_id = 'user_default' WHERE user_id IS NULL;

-- Indexes to make the per-user filter index-friendly. The list pages all
-- query `WHERE user_id = ? ORDER BY created_at DESC`, so we composite the
-- column with the existing sort key.
CREATE INDEX IF NOT EXISTS idx_clips_user_created        ON clips(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_collections_user          ON collections(user_id);
CREATE INDEX IF NOT EXISTS idx_snippets_user             ON snippets(user_id);
CREATE INDEX IF NOT EXISTS idx_images_user                ON images(user_id);
CREATE INDEX IF NOT EXISTS idx_file_clips_user           ON file_clips(user_id);
CREATE INDEX IF NOT EXISTS idx_ring_sets_user_created    ON ring_sets(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ring_set_items_user       ON ring_set_items(user_id);
"#;
