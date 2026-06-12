//! SQLite + FTS5 storage layer.
//!
//! Opens a WAL-mode database with a small connection pool and applies migrations on startup.

pub mod migrations;
pub mod repo;

use std::path::Path;

use anyhow::Context;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = r2d2::PooledConnection<SqliteConnectionManager>;

/// Open a connection pool to the database at `path`, applying the per-connection pragmas
/// that keep ClipVault fast (WAL, NORMAL sync, mmap, large cache).
pub fn open_pool(path: &Path) -> anyhow::Result<DbPool> {
    let manager = SqliteConnectionManager::file(path)
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)
        .with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA temp_store = MEMORY;
                 PRAGMA mmap_size = 30000000000;
                 PRAGMA cache_size = -64000;
                 PRAGMA page_size = 4096;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;",
            )
        });

    let pool = Pool::builder()
        .max_size(8)
        .min_idle(Some(1))
        .build(manager)
        .context("failed to build SQLite connection pool")?;

    // Probe the connection to ensure the file is reachable.
    {
        let _conn = pool.get().context("failed to acquire initial connection")?;
    }

    Ok(pool)
}
