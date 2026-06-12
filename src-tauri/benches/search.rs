//! Benchmarks for the ClipVault search engine.
//!
//! Run with: `cargo bench --bench search` from the `src-tauri` directory.

use criterion::{criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use std::time::Instant;

fn setup_db(n: usize) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;
         PRAGMA cache_size = -64000;
         PRAGMA page_size = 4096;
         CREATE VIRTUAL TABLE clips_fts USING fts5(text_preview, tokenize='unicode61 remove_diacritics 2');
         CREATE TABLE clips (id INTEGER PRIMARY KEY, text_preview TEXT, created_at INTEGER);
         CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
           INSERT INTO clips_fts(rowid, text_preview) VALUES (new.rowid, new.text_preview);
         END;",
    )
    .unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..n {
        tx.execute(
            "INSERT INTO clips (id, text_preview, created_at) VALUES (?, ?, ?)",
            rusqlite::params![i as i64, format!("sample text {} with words like server ubuntu password github", i), 1700000000000 + i as i64],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    conn
}

fn bench_search(c: &mut Criterion) {
    let conn = setup_db(100_000);
    c.bench_function("search_100k", |b| {
        b.iter(|| {
            let start = Instant::now();
            let mut stmt = conn
                .prepare("SELECT id FROM clips_fts WHERE clips_fts MATCH ?")
                .unwrap();
            let rows: Vec<i64> = stmt
                .query_map(["server OR ubuntu OR password"], |r| r.get(0))
                .unwrap()
                .filter_map(Result::ok)
                .collect();
            (rows.len(), start.elapsed())
        })
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
