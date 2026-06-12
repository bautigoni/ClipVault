//! Versioned SQLite migrations. Applied on every startup; each migration is a single SQL batch
//! stored in its own file. Add a new `Migration` entry and bump `CURRENT_VERSION` to upgrade
//! the schema.

use anyhow::Context;
use rusqlite::params;
use tracing::info;

use super::{DbConn, DbPool};

#[allow(dead_code)]
const CURRENT_VERSION: i32 = 2;

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
