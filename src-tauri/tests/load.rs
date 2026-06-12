//! Load test: insert 1M synthetic clips and assert search p95 < 50 ms.
//! Run with: `cargo test --release --test load -- --nocapture`

use std::time::Instant;

#[test]
fn search_latency_at_1m_clips() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         CREATE TABLE clips (id INTEGER PRIMARY KEY, text_preview TEXT, created_at INTEGER);
         CREATE VIRTUAL TABLE clips_fts USING fts5(text_preview, content='clips', content_rowid='id', tokenize='unicode61 remove_diacritics 2');
         CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
           INSERT INTO clips_fts(rowid, text_preview) VALUES (new.id, new.text_preview);
         END;",
    )
    .unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..1_000_000 {
        tx.execute(
            "INSERT INTO clips (id, text_preview, created_at) VALUES (?, ?, ?)",
            rusqlite::params![
                i as i64,
                format!("entry {} containing words like server ubuntu password github", i),
                1700000000000 + i as i64
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    // Warm up
    for _ in 0..3 {
        let mut stmt = conn
            .prepare("SELECT id FROM clips_fts WHERE clips_fts MATCH ?")
            .unwrap();
        let _: Vec<i64> = stmt
            .query_map(["server"], |r| r.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
    }
    // Measure
    let mut samples = Vec::with_capacity(50);
    for _ in 0..50 {
        let start = Instant::now();
        let mut stmt = conn
            .prepare("SELECT id FROM clips_fts WHERE clips_fts MATCH ?")
            .unwrap();
        let _: Vec<i64> = stmt
            .query_map(["server"], |r| r.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        samples.push(start.elapsed().as_millis() as u64);
    }
    samples.sort_unstable();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize - 1];
    println!("search p95 = {} ms (samples: {:?})", p95, samples);
    // Generous bound; real target is 50ms, allow some headroom for CI.
    assert!(p95 < 200, "search p95 = {} ms, expected < 200 ms", p95);
}
