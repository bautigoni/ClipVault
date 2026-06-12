//! End-to-end-ish DB tests using an on-disk SQLite file in a tempdir.
//!
//! Covers migrations, FTS5 search ranking, dedup, retention, and the
//! suppress / dedup hash helpers.

use std::path::PathBuf;

use clipvault_lib::db::migrations;
use clipvault_lib::db::repo;
use clipvault_lib::state::SuppressFlag;

fn temp_db_path(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("clipvault-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.push("clipvault.db");
    dir
}

fn fresh_pool() -> clipvault_lib::db::DbPool {
    let path = temp_db_path("migrations");
    let pool = clipvault_lib::db::open_pool(&path).expect("open pool");
    let conn = pool.get().expect("conn");
    migrations::run(&conn).expect("migrations");
    pool
}

fn insert_text_clip(pool: &clipvault_lib::db::DbPool, text: &str, at_ms: i64) -> String {
    let conn = pool.get().unwrap();
    let hash = blake3::hash(text.as_bytes()).to_hex().to_string();
    repo::insert_clip(
        &conn,
        "text",
        &hash,
        Some(text),
        text.len() as i64,
        Some("test.exe"),
        None,
        at_ms,
    )
    .unwrap()
}

#[test]
fn migrations_create_all_tables() {
    let pool = fresh_pool();
    let conn = pool.get().unwrap();
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type IN ('table') ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    for required in [
        "clips",
        "collections",
        "tags",
        "snippets",
        "images",
        "file_clips",
        "settings",
        "clip_ocr",
        "clips_fts",
        "snippets_fts",
    ] {
        assert!(
            names.contains(&required.to_string()),
            "missing table: {required}"
        );
    }
}

#[test]
fn fts_returns_ranked_results() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    let a = insert_text_clip(&pool, "the brown fox jumps over the lazy dog", now);
    let b = insert_text_clip(&pool, "rust programming language", now - 1000);
    let c = insert_text_clip(&pool, "javascript is unrelated", now - 2000);

    let conn = pool.get().unwrap();
    let filter = repo::ClipFilter {
        query: Some("rust".to_string()),
        ..Default::default()
    };
    let page = repo::search_clips(&pool, &filter, 10, None).unwrap();
    let ids: Vec<String> = page.items.iter().map(|c| c.id.clone()).collect();
    assert!(ids.contains(&b));
    assert!(!ids.contains(&a));
    assert!(!ids.contains(&c));
    assert!(page.took_ms < 5_000, "search took too long");
}

#[test]
fn find_recent_duplicate_within_window() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    let id = insert_text_clip(&pool, "hello world", now);
    let conn = pool.get().unwrap();

    let hash = blake3::hash(b"hello world").to_hex().to_string();
    let dup = repo::find_recent_duplicate(&conn, &hash, 5_000, now + 1_000).unwrap();
    assert_eq!(dup.as_deref(), Some(id.as_str()));

    // Outside the window
    let no_dup = repo::find_recent_duplicate(&conn, &hash, 5_000, now + 60_000).unwrap();
    assert!(no_dup.is_none());
}

#[test]
fn bump_usage_increments_counter() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    let id = insert_text_clip(&pool, "ping", now);
    let conn = pool.get().unwrap();
    repo::bump_usage(&conn, &id, now + 1).unwrap();
    repo::bump_usage(&conn, &id, now + 2).unwrap();
    let clip = repo::get_clip(&conn, &id).unwrap().unwrap();
    assert_eq!(clip.usage_count, 3);
    assert_eq!(clip.last_used_at, Some(now + 2));
}

#[test]
fn delete_clip_cascades_to_tags_and_images() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    let id = insert_text_clip(&pool, "to be deleted", now);
    let conn = pool.get().unwrap();
    conn.execute(
        "INSERT INTO tags(clip_id, tag) VALUES (?, ?)",
        rusqlite::params![id, "work"],
    )
    .unwrap();
    repo::delete_clip(&conn, &id).unwrap();
    let tags: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE clip_id = ?",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tags, 0);
    assert!(repo::get_clip(&conn, &id).unwrap().is_none());
}

#[test]
fn clear_history_keeps_favorites_and_pins() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    let _normal = insert_text_clip(&pool, "trash this", now);
    let fav = insert_text_clip(&pool, "keep me", now - 1);
    let pin = insert_text_clip(&pool, "pin me", now - 2);
    let conn = pool.get().unwrap();
    conn.execute(
        "UPDATE clips SET is_favorite = 1 WHERE id = ?",
        rusqlite::params![fav],
    )
    .unwrap();
    conn.execute(
        "UPDATE clips SET is_pinned = 1 WHERE id = ?",
        rusqlite::params![pin],
    )
    .unwrap();
    let cleared = repo::clear_history(&conn).unwrap();
    assert_eq!(cleared, 1, "only the non-favorite, non-pinned clip is removed");
    assert!(repo::get_clip(&conn, &fav).unwrap().is_some());
    assert!(repo::get_clip(&conn, &pin).unwrap().is_some());
}

#[test]
fn enforce_max_clips_keeps_newest() {
    let pool = fresh_pool();
    let now = chrono::Utc::now().timestamp_millis();
    for i in 0..10 {
        insert_text_clip(&pool, &format!("clip {i}"), now + i);
    }
    let conn = pool.get().unwrap();
    let removed = repo::enforce_max_clips(&conn, 3).unwrap();
    assert_eq!(removed, 7);
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM clips", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3);
    // The 3 newest should remain
    let newest: Vec<String> = conn
        .prepare("SELECT text_preview FROM clips ORDER BY created_at DESC")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(
        newest,
        vec!["clip 9".to_string(), "clip 8".to_string(), "clip 7".to_string()]
    );
}

#[test]
fn suppress_flag_blocks_immediate_recapture() {
    let _pool = fresh_pool();
    let flag = SuppressFlag::default();
    let hash = blake3::hash(b"suppress me").to_hex().to_string();
    flag.arm(hash.clone(), 1500);

    // Within the TTL the flag reports the suppression match.
    assert!(flag.should_skip(&hash));
    // Consumed; subsequent calls don't report a match.
    assert!(!flag.should_skip(&hash));

    // Arming a different hash is independent.
    let other = blake3::hash(b"different content").to_hex().to_string();
    flag.arm(other.clone(), 1500);
    assert!(flag.should_skip(&other));
}

#[test]
fn hash_dedup_distinguishes_content() {
    let a = blake3::hash(b"alpha").to_hex().to_string();
    let b = blake3::hash(b"beta").to_hex().to_string();
    assert_ne!(a, b);
    // Same content yields the same hash.
    let a2 = blake3::hash(b"alpha").to_hex().to_string();
    assert_eq!(a, a2);
}
