//! Repository functions that talk to SQLite. All async work goes through the r2d2 pool.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{DbConn, DbPool};
use crate::ring::buffer::SlotData;
use crate::state::{Clip, Collection, ImageMeta, SearchPage, Snippet};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipFilter {
    pub query: Option<String>,
    pub kind: Option<String>,
    pub source_app: Option<String>,
    pub collection_id: Option<String>,
    pub favorites_only: bool,
    pub pinned_only: bool,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipPatch {
    pub is_favorite: Option<bool>,
    pub is_pinned: Option<bool>,
    pub collection_id: Option<Option<String>>, // outer = set, inner = Some/None
    pub tags: Option<Vec<String>>,
    pub text_preview: Option<String>,
}

pub fn insert_clip(
    conn: &DbConn,
    kind: &str,
    content_hash: &str,
    text_preview: Option<&str>,
    byte_size: i64,
    source_app: Option<&str>,
    source_title: Option<&str>,
    now_ms: i64,
    user_id: &str,
) -> anyhow::Result<String> {
    let id = Ulid::new().to_string();
    conn.execute(
        "INSERT INTO clips
         (id, type, content_hash, text_preview, byte_size, source_app, source_title, created_at, updated_at, user_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![id, kind, content_hash, text_preview, byte_size, source_app, source_title, now_ms, now_ms, user_id],
    )?;
    Ok(id)
}

pub fn find_recent_duplicate(
    conn: &DbConn,
    content_hash: &str,
    within_ms: i64,
    now_ms: i64,
) -> anyhow::Result<Option<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id FROM clips
         WHERE content_hash = ? AND created_at >= ?
         ORDER BY created_at DESC LIMIT 1",
    )?;
    let id: Option<String> = stmt
        .query_row(params![content_hash, now_ms - within_ms], |row| row.get(0))
        .optional()?;
    Ok(id)
}

pub fn bump_usage(conn: &DbConn, id: &str, now_ms: i64) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE clips SET usage_count = usage_count + 1, last_used_at = ? WHERE id = ?",
        params![now_ms, id],
    )?;
    Ok(())
}

pub fn get_clip(conn: &DbConn, id: &str) -> anyhow::Result<Option<Clip>> {
    hydrate_clip(conn, "WHERE clips.id = ?", params![id], None)
}

pub fn delete_clip(conn: &DbConn, id: &str) -> anyhow::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM tags WHERE clip_id = ?", params![id])?;
    tx.execute("DELETE FROM images WHERE clip_id = ?", params![id])?;
    tx.execute("DELETE FROM file_clips WHERE clip_id = ?", params![id])?;
    tx.execute("DELETE FROM clip_ocr WHERE clip_id = ?", params![id])?;
    tx.execute("DELETE FROM clips WHERE id = ?", params![id])?;
    tx.commit()?;
    Ok(())
}

pub fn clear_history(conn: &DbConn) -> anyhow::Result<usize> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM tags", [])?;
    tx.execute("DELETE FROM images", [])?;
    tx.execute("DELETE FROM file_clips", [])?;
    tx.execute("DELETE FROM clip_ocr", [])?;
    // The `clips_ad` trigger in the initial migration keeps clips_fts in
    // sync, so a plain DELETE is enough.
    let count = tx.execute("DELETE FROM clips WHERE is_favorite = 0 AND is_pinned = 0", [])?;
    tx.commit()?;
    Ok(count)
}

pub fn apply_patch(conn: &DbConn, id: &str, patch: &ClipPatch) -> anyhow::Result<()> {
    let tx = conn.unchecked_transaction()?;
    if let Some(fav) = patch.is_favorite {
        tx.execute("UPDATE clips SET is_favorite = ?, updated_at = ? WHERE id = ?", params![fav as i64, now_ms(), id])?;
    }
    if let Some(pin) = patch.is_pinned {
        let pinned_at = if pin { Some(now_ms()) } else { None };
        tx.execute(
            "UPDATE clips SET is_pinned = ?, pinned_at = ?, updated_at = ? WHERE id = ?",
            params![pin as i64, pinned_at, now_ms(), id],
        )?;
    }
    if let Some(collection) = &patch.collection_id {
        tx.execute(
            "UPDATE clips SET collection_id = ?, updated_at = ? WHERE id = ?",
            params![collection.as_deref(), now_ms(), id],
        )?;
    }
    if let Some(text) = &patch.text_preview {
        // FTS index is kept in sync by the `clips_au` trigger installed in the
        // initial migration, so a plain UPDATE here is enough.
        tx.execute(
            "UPDATE clips SET text_preview = ?, updated_at = ? WHERE id = ?",
            params![text, now_ms(), id],
        )?;
    }
    if let Some(tags) = &patch.tags {
        tx.execute("DELETE FROM tags WHERE clip_id = ?", params![id])?;
        for tag in tags {
            let t = tag.trim();
            if t.is_empty() {
                continue;
            }
            tx.execute(
                "INSERT OR IGNORE INTO tags (clip_id, tag) VALUES (?, ?)",
                params![id, t],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn list_collections(conn: &DbConn) -> anyhow::Result<Vec<Collection>> {
    let mut stmt = conn.prepare_cached(
        "SELECT c.id, c.name, c.icon, c.created_at, COUNT(cl.id)
         FROM collections c
         LEFT JOIN clips cl ON cl.collection_id = c.id
         GROUP BY c.id
         ORDER BY c.name",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                created_at: row.get(3)?,
                clip_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
// Clipboard Ring helpers
// ---------------------------------------------------------------------------

/// Lightweight row used to populate the in-memory ring buffer.
fn row_to_slot(row: &rusqlite::Row<'_>) -> rusqlite::Result<SlotData> {
    let kind: String = row.get(2)?;
    let preview: Option<String> = row.get(1)?;
    let is_pinned: i64 = row.get(4)?;
    let is_favorite: i64 = row.get(5)?;
    Ok(SlotData {
        id: row.get(0)?,
        kind,
        preview,
        is_pinned: is_pinned != 0,
        is_favorite: is_favorite != 0,
    })
}

/// Build a `WHERE` clause fragment that excludes kinds the ring config rejects.
fn ring_kind_filter(config: &crate::ring::buffer::RingConfig) -> String {
    let mut excluded: Vec<&'static str> = Vec::new();
    if !config.include_files {
        excluded.push("'files'");
    }
    if !config.include_images {
        excluded.push("'image'");
    }
    if excluded.is_empty() {
        String::new()
    } else {
        format!(" AND type NOT IN ({})", excluded.join(","))
    }
}

/// List the most recent clips, respecting the ring config's kind and
/// sensitivity filters.
pub fn list_recent(
    state: &crate::state::AppState,
    config: &crate::ring::buffer::RingConfig,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let kind_filter = ring_kind_filter(config);
    let sensitive = if config.include_sensitive { "" } else { " AND is_sensitive = 0" };
    let sql = format!(
        "SELECT id, text_preview, type, byte_size, is_pinned, is_favorite
         FROM clips
         WHERE 1=1{sensitive}{kind_filter}
         ORDER BY is_pinned DESC, created_at DESC, id DESC
         LIMIT ?"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let rows = stmt
        .query_map(params![config.capacity as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List recent clips that are favorites.
pub fn list_recent_favorites(
    state: &crate::state::AppState,
    limit: usize,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT id, text_preview, type, byte_size, is_pinned, is_favorite
         FROM clips
         WHERE is_favorite = 1
         ORDER BY last_used_at DESC, created_at DESC, id DESC
         LIMIT ?",
    )?;
    let rows = stmt
        .query_map(params![limit as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List recent clips in a given collection.
pub fn list_recent_by_collection(
    state: &crate::state::AppState,
    collection_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT id, text_preview, type, byte_size, is_pinned, is_favorite
         FROM clips
         WHERE collection_id = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![collection_id, limit as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List recent clips from a particular source app (exe name).
pub fn list_recent_by_source_app(
    state: &crate::state::AppState,
    exe: &str,
    limit: usize,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT id, text_preview, type, byte_size, is_pinned, is_favorite
         FROM clips
         WHERE source_app = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![exe, limit as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List recent clips of a given kind.
pub fn list_recent_by_kind(
    state: &crate::state::AppState,
    kind: &str,
    limit: usize,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT id, text_preview, type, byte_size, is_pinned, is_favorite
         FROM clips
         WHERE type = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![kind, limit as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List the items in a named ring set, in the order defined by `position`.
pub fn list_ring_set_items(
    state: &crate::state::AppState,
    set_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<SlotData>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT c.id, c.text_preview, c.type, c.byte_size, c.is_pinned, c.is_favorite
         FROM ring_set_items r
         JOIN clips c ON c.id = r.clip_id
         WHERE r.set_id = ?1
         ORDER BY r.position ASC, r.clip_id ASC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![set_id, limit as i64], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ----- Ring set CRUD (P1) -------------------------------------------------

/// A persisted ring working set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSet {
    pub id: String,
    pub name: String,
    pub scope_kind: String,
    pub scope_ref: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub item_count: i64,
}

pub fn list_ring_sets(state: &crate::state::AppState) -> anyhow::Result<Vec<RingSet>> {
    let conn = state.db.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT s.id, s.name, s.scope_kind, s.scope_ref, s.created_at, s.updated_at,
                COALESCE((SELECT COUNT(*) FROM ring_set_items WHERE set_id = s.id), 0)
         FROM ring_sets s ORDER BY s.name",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(RingSet {
                id: row.get(0)?,
                name: row.get(1)?,
                scope_kind: row.get(2)?,
                scope_ref: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                item_count: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_ring_set(
    state: &crate::state::AppState,
    name: &str,
    scope_kind: &str,
    scope_ref: Option<&str>,
) -> anyhow::Result<RingSet> {
    let conn = state.db.get()?;
    let id = Ulid::new().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO ring_sets (id, name, scope_kind, scope_ref, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, name, scope_kind, scope_ref, now, now],
    )?;
    Ok(RingSet {
        id,
        name: name.to_string(),
        scope_kind: scope_kind.to_string(),
        scope_ref: scope_ref.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
        item_count: 0,
    })
}

pub fn delete_ring_set(state: &crate::state::AppState, id: &str) -> anyhow::Result<()> {
    let conn = state.db.get()?;
    conn.execute("DELETE FROM ring_sets WHERE id = ?", params![id])?;
    Ok(())
}

pub fn add_to_ring_set(
    state: &crate::state::AppState,
    set_id: &str,
    clip_id: &str,
    position: i64,
) -> anyhow::Result<()> {
    let conn = state.db.get()?;
    let now = now_ms();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR REPLACE INTO ring_set_items (set_id, clip_id, position) VALUES (?, ?, ?)",
        params![set_id, clip_id, position],
    )?;
    tx.execute(
        "UPDATE ring_sets SET updated_at = ? WHERE id = ?",
        params![now, set_id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn remove_from_ring_set(
    state: &crate::state::AppState,
    set_id: &str,
    clip_id: &str,
) -> anyhow::Result<()> {
    let conn = state.db.get()?;
    conn.execute(
        "DELETE FROM ring_set_items WHERE set_id = ? AND clip_id = ?",
        params![set_id, clip_id],
    )?;
    Ok(())
}

pub fn create_collection(conn: &DbConn, name: &str, icon: Option<&str>) -> anyhow::Result<Collection> {
    let id = Ulid::new().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO collections (id, name, icon, created_at) VALUES (?, ?, ?, ?)",
        params![id, name, icon, now],
    )?;
    Ok(Collection {
        id,
        name: name.to_string(),
        icon: icon.map(|s| s.to_string()),
        created_at: now,
        clip_count: 0,
    })
}

pub fn delete_collection(conn: &DbConn, id: &str) -> anyhow::Result<()> {
    conn.execute("UPDATE clips SET collection_id = NULL WHERE collection_id = ?", params![id])?;
    conn.execute("DELETE FROM collections WHERE id = ?", params![id])?;
    Ok(())
}

pub fn rename_collection(conn: &DbConn, id: &str, name: &str, icon: Option<&str>) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE collections SET name = ?, icon = ? WHERE id = ?",
        params![name, icon, id],
    )?;
    Ok(())
}

pub fn list_snippets(conn: &DbConn) -> anyhow::Result<Vec<Snippet>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, language, body, is_favorite, created_at, updated_at
         FROM snippets ORDER BY is_favorite DESC, updated_at DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                title: row.get(1)?,
                language: row.get(2)?,
                body: row.get(3)?,
                is_favorite: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn search_snippets(conn: &DbConn, query: &str, limit: usize) -> anyhow::Result<Vec<Snippet>> {
    let q = query.trim();
    if q.is_empty() {
        return list_snippets(conn);
    }

    // Try FTS5 first.
    let mut fts_q = String::with_capacity(q.len() + 2);
    for tok in q.split_whitespace() {
        let cleaned: String = tok
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        fts_q.push('"');
        fts_q.push_str(&cleaned);
        fts_q.push('"');
        fts_q.push(' ');
    }

    if !fts_q.is_empty() {
        let mut stmt = conn.prepare_cached(
            "SELECT s.id, s.title, s.language, s.body, s.is_favorite, s.created_at, s.updated_at
             FROM snippets_fts
             JOIN snippets s ON s.rowid = snippets_fts.rowid
             WHERE snippets_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![fts_q.trim(), limit as i64], |row| {
                Ok(Snippet {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    language: row.get(2)?,
                    body: row.get(3)?,
                    is_favorite: row.get::<_, i64>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        if !rows.is_empty() {
            return Ok(rows);
        }
    }

    // Substring fallback.
    let like = format!("%{}%", q);
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, language, body, is_favorite, created_at, updated_at
         FROM snippets
         WHERE title LIKE ?1 COLLATE NOCASE
            OR body  LIKE ?1 COLLATE NOCASE
         ORDER BY is_favorite DESC, updated_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![like, limit as i64], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                title: row.get(1)?,
                language: row.get(2)?,
                body: row.get(3)?,
                is_favorite: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upsert_snippet(
    conn: &DbConn,
    id: Option<&str>,
    title: &str,
    language: &str,
    body: &str,
    is_favorite: bool,
) -> anyhow::Result<Snippet> {
    let now = now_ms();
    let new_id = match id {
        Some(existing) => {
            conn.execute(
                "UPDATE snippets SET title = ?, language = ?, body = ?, is_favorite = ?, updated_at = ?
                 WHERE id = ?",
                params![title, language, body, is_favorite as i64, now, existing],
            )?;
            existing.to_string()
        }
        None => {
            let new_id = Ulid::new().to_string();
            conn.execute(
                "INSERT INTO snippets (id, title, language, body, is_favorite, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![new_id, title, language, body, is_favorite as i64, now, now],
            )?;
            new_id
        }
    };
    Ok(Snippet {
        id: new_id,
        title: title.to_string(),
        language: language.to_string(),
        body: body.to_string(),
        is_favorite,
        created_at: id.map(|_| now).unwrap_or(now),
        updated_at: now,
    })
}

pub fn delete_snippet(conn: &DbConn, id: &str) -> anyhow::Result<()> {
    conn.execute("DELETE FROM snippets WHERE id = ?", params![id])?;
    Ok(())
}

pub fn get_snippet_body(conn: &DbConn, id: &str) -> anyhow::Result<Option<String>> {
    let body: Option<String> = conn
        .query_row(
            "SELECT body FROM snippets WHERE id = ?",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(body)
}

pub fn attach_image(
    conn: &DbConn,
    clip_id: &str,
    path: &str,
    thumb_path: &str,
    width: i64,
    height: i64,
    mime: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO images (clip_id, path, width, height, thumb_path, mime) VALUES (?, ?, ?, ?, ?, ?)",
        params![clip_id, path, width, height, thumb_path, mime],
    )?;
    Ok(())
}

pub fn attach_files(conn: &DbConn, clip_id: &str, paths_json: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO file_clips (clip_id, paths) VALUES (?, ?)",
        params![clip_id, paths_json],
    )?;
    Ok(())
}

/// Persist OCR'd text for an image clip. Replaces any previous OCR result for
/// the same clip; we treat OCR as a one-shot overwrite per re-run, not a log.
pub fn save_ocr(
    conn: &DbConn,
    clip_id: &str,
    text: &str,
    now_ms: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO clip_ocr (clip_id, text, updated_at) VALUES (?, ?, ?)",
        params![clip_id, text, now_ms],
    )?;
    Ok(())
}

/// Read back the OCR text for a clip, if any was ever written. Returns
/// `Ok(None)` when the clip has no OCR row — distinct from the
/// "OCR ran and produced empty text" case the caller has to handle.
pub fn load_ocr(conn: &DbConn, clip_id: &str) -> anyhow::Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT text FROM clip_ocr WHERE clip_id = ?")?;
    let text: Option<String> = stmt
        .query_row(params![clip_id], |r| r.get(0))
        .ok()
        .flatten();
    Ok(text)
}

/// Append a single event to the activity log. We don't ever log content,
/// only metadata (clip_id, source app, action kind, free-form detail) — see
/// the schema comment for the safety story.
pub fn log_activity(
    conn: &DbConn,
    kind: &str,
    clip_id: Option<&str>,
    source_app: Option<&str>,
    detail: Option<&str>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO activity_log (ts_ms, kind, clip_id, source_app, detail) VALUES (?, ?, ?, ?, ?)",
        params![now_ms(), kind, clip_id, source_app, detail],
    )?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct ActivityEntry {
    pub id: i64,
    pub ts_ms: i64,
    pub kind: String,
    pub clip_id: Option<String>,
    pub source_app: Option<String>,
    pub detail: Option<String>,
}

/// Read the most recent `limit` activity entries, newest first.
pub fn list_activity(conn: &DbConn, limit: i64) -> anyhow::Result<Vec<ActivityEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, ts_ms, kind, clip_id, source_app, detail
         FROM activity_log ORDER BY id DESC LIMIT ?",
    )?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(ActivityEntry {
                id: r.get(0)?,
                ts_ms: r.get(1)?,
                kind: r.get(2)?,
                clip_id: r.get(3)?,
                source_app: r.get(4)?,
                detail: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Hard-truncate the activity log. Used from settings ("Clear activity")
/// and from a test fixture. We intentionally don't return a count — the
/// caller doesn't need it.
pub fn clear_activity(conn: &DbConn) -> anyhow::Result<()> {
    conn.execute("DELETE FROM activity_log", [])?;
    Ok(())
}

pub fn list_tags(conn: &DbConn) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT tag, COUNT(*) AS c FROM tags GROUP BY tag ORDER BY c DESC, tag LIMIT 200",
    )?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn list_source_apps(conn: &DbConn) -> anyhow::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT source_app, COUNT(*) AS c FROM clips
         WHERE source_app IS NOT NULL AND source_app <> ''
         GROUP BY source_app ORDER BY c DESC LIMIT 100",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// search_clips, hydrate_clip, now_ms, enforce_max_clips
// ---------------------------------------------------------------------------

/// Current epoch time in milliseconds.
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Hydrate a `Clip` row, joining the collections table for `collection_name`.
/// Used by both `get_clip` and `search_clips` to keep behaviour in sync.
pub fn hydrate_clip(
    conn: &Connection,
    where_clause: &str,
    params: impl rusqlite::Params,
    extra_select: Option<&str>,
) -> anyhow::Result<Option<Clip>> {
    let extra = extra_select.unwrap_or("");
    let sql = format!(
        "SELECT clips.id, clips.type, clips.content_hash, clips.text_preview, clips.byte_size,
                clips.source_app, clips.source_title, clips.is_favorite, clips.is_pinned,
                clips.is_sensitive, clips.collection_id, clips.created_at, clips.updated_at,
                clips.usage_count, clips.last_used_at, clips.pinned_at, collections.name{extra}
         FROM clips
         LEFT JOIN collections ON collections.id = clips.collection_id
         {where_clause}",
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(params)?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_clip(conn, row, extra_select.is_some())?))
    } else {
        Ok(None)
    }
}

/// Public helper used by `fuzzy_fallback` in the search module. The caller
/// passes the `has_total_count` flag to indicate whether a trailing
/// `COUNT(*) OVER ()` column is present (we ignore it; the field is unused
/// here because we hydrate the full clip via `get_clip_inner`).
pub fn row_to_clip(
    conn: &Connection,
    row: &rusqlite::Row<'_>,
    has_total_count: bool,
) -> rusqlite::Result<Clip> {
    let id: String = row.get(0)?;
    let _ = has_total_count; // declared and intentionally unused at the row level
    get_clip_inner(conn, &id)
        .map_err(|_e| rusqlite::Error::InvalidQuery)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

fn get_clip_inner(conn: &Connection, id: &str) -> anyhow::Result<Option<Clip>> {
    let mut stmt = conn.prepare_cached(
        "SELECT clips.id, clips.type, clips.content_hash, clips.text_preview, clips.byte_size,
                clips.source_app, clips.source_title, clips.is_favorite, clips.is_pinned,
                clips.is_sensitive, clips.collection_id, clips.created_at, clips.updated_at,
                clips.usage_count, clips.last_used_at, clips.pinned_at, collections.name,
                (SELECT GROUP_CONCAT(tag, '\u{1f}') FROM tags WHERE clip_id = clips.id)
         FROM clips
         LEFT JOIN collections ON collections.id = clips.collection_id
         WHERE clips.id = ?",
    )?;
    let mut rows = stmt.query(params![id])?;
    let Some(row) = rows.next()? else { return Ok(None); };
    let id_value: String = row.get(0)?;
    let mut clip = decode_clip_row(&id_value, row)?;
    clip.image = load_image_meta(conn, &id_value)?;
    clip.file_paths = load_file_paths(conn, &id_value)?;
    Ok(Some(clip))
}

fn decode_clip_row(id: &str, row: &rusqlite::Row<'_>) -> anyhow::Result<Clip> {
    let kind: String = row.get(1)?;
    let is_favorite: i64 = row.get(7)?;
    let is_pinned: i64 = row.get(8)?;
    let is_sensitive: i64 = row.get(9)?;
    let collection_name: Option<String> = row.get(16)?;
    let tags_joined: Option<String> = row.get(17)?;
    let tags = tags_joined
        .map(|s| s.split('\u{1f}').filter(|t| !t.is_empty()).map(String::from).collect())
        .unwrap_or_default();

    Ok(Clip {
        id: id.to_string(),
        kind,
        content_hash: row.get(2)?,
        text_preview: row.get(3)?,
        byte_size: row.get(4)?,
        source_app: row.get(5)?,
        source_title: row.get(6)?,
        is_favorite: is_favorite != 0,
        is_pinned: is_pinned != 0,
        is_sensitive: is_sensitive != 0,
        collection_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        usage_count: row.get(13)?,
        last_used_at: row.get(14)?,
        pinned_at: row.get(15)?,
        tags,
        image: None,
        file_paths: None,
        collection_name,
    })
}

fn load_image_meta(conn: &Connection, clip_id: &str) -> anyhow::Result<Option<ImageMeta>> {
    let mut stmt = conn.prepare_cached(
        "SELECT path, thumb_path, width, height, mime
         FROM images WHERE clip_id = ?",
    )?;
    let mut rows = stmt.query(params![clip_id])?;
    let Some(row) = rows.next()? else { return Ok(None); };
    Ok(Some(ImageMeta {
        path: row.get(0)?,
        thumb_path: row.get(1)?,
        width: row.get(2)?,
        height: row.get(3)?,
        mime: row.get(4)?,
    }))
}

fn load_file_paths(conn: &Connection, clip_id: &str) -> anyhow::Result<Option<Vec<String>>> {
    let mut stmt = conn.prepare_cached("SELECT paths FROM file_clips WHERE clip_id = ?")?;
    let mut rows = stmt.query(params![clip_id])?;
    let Some(row) = rows.next()? else { return Ok(None); };
    let paths_json: String = row.get(0)?;
    let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
    Ok(Some(paths))
}

/// Cursor-based search used by the timeline + search UI. Returns a `SearchPage`
/// with the next cursor for the next page, or `None` if exhausted.
pub fn search_clips(
    pool: &DbPool,
    filter: &ClipFilter,
    limit: usize,
    cursor: Option<&str>,
) -> anyhow::Result<SearchPage> {
    use std::time::Instant;
    let start = Instant::now();
    let conn = pool.get()?;

    let mut where_clauses: Vec<String> = Vec::new();
    let mut bind: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = filter.query.as_ref().filter(|q| !q.trim().is_empty()) {
        let mut fts_q = String::new();
        for tok in q.split_whitespace() {
            let cleaned: String = tok
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if cleaned.is_empty() {
                continue;
            }
            fts_q.push('"');
            fts_q.push_str(&cleaned);
            fts_q.push('"');
            fts_q.push(' ');
        }
        if !fts_q.is_empty() {
            where_clauses.push(
                "clips.id IN (SELECT rowid FROM clips_fts WHERE clips_fts MATCH ?)".to_string(),
            );
            bind.push(Box::new(fts_q.trim().to_string()));
        }
    }
    if let Some(kind) = filter.kind.as_ref() {
        where_clauses.push("clips.type = ?".to_string());
        bind.push(Box::new(kind.clone()));
    }
    if let Some(app) = filter.source_app.as_ref() {
        where_clauses.push("clips.source_app = ?".to_string());
        bind.push(Box::new(app.clone()));
    }
    if let Some(cid) = filter.collection_id.as_ref() {
        where_clauses.push("clips.collection_id = ?".to_string());
        bind.push(Box::new(cid.clone()));
    }
    if filter.favorites_only {
        where_clauses.push("clips.is_favorite = 1".to_string());
    }
    if filter.pinned_only {
        where_clauses.push("clips.is_pinned = 1".to_string());
    }
    if let Some(since) = filter.since {
        where_clauses.push("clips.created_at >= ?".to_string());
        bind.push(Box::new(since));
    }
    if let Some(until) = filter.until {
        where_clauses.push("clips.created_at < ?".to_string());
        bind.push(Box::new(until));
    }
    if let Some(tag) = filter.tag.as_ref() {
        where_clauses.push(
            "clips.id IN (SELECT clip_id FROM tags WHERE tag = ?)".to_string(),
        );
        bind.push(Box::new(tag.clone()));
    }
    if let Some(c) = cursor {
        where_clauses.push("clips.created_at < ?".to_string());
        bind.push(Box::new(c.parse::<i64>().unwrap_or(0)));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Use COUNT(*) OVER () to get the true total in the same query. Image
    // and file_clips metadata are joined as LEFT JOINs so we don't have to
    // delimit or parse the values.
    //
    // Column layout (zero-indexed):
    //   0  clips.id
    //   1  clips.type
    //   2  clips.content_hash
    //   3  clips.text_preview
    //   4  clips.byte_size
    //   5  clips.source_app
    //   6  clips.source_title
    //   7  clips.is_favorite
    //   8  clips.is_pinned
    //   9  clips.is_sensitive
    //  10  clips.collection_id
    //  11  clips.created_at
    //  12  clips.updated_at
    //  13  clips.usage_count
    //  14  clips.last_used_at
    //  15  clips.pinned_at
    //  16  collections.name
    //  17  tags_joined
    //  18  images.path
    //  19  images.thumb_path
    //  20  images.width
    //  21  images.height
    //  22  images.mime
    //  23  file_clips.paths
    //  24  COUNT(*) OVER ()
    let sql = format!(
        "SELECT clips.id, clips.type, clips.content_hash, clips.text_preview, clips.byte_size,
                clips.source_app, clips.source_title, clips.is_favorite, clips.is_pinned,
                clips.is_sensitive, clips.collection_id, clips.created_at, clips.updated_at,
                clips.usage_count, clips.last_used_at, clips.pinned_at, collections.name,
                (SELECT GROUP_CONCAT(tag, '\u{1f}') FROM tags WHERE clip_id = clips.id),
                images.path, images.thumb_path, images.width, images.height, images.mime,
                file_clips.paths,
                COUNT(*) OVER () AS total_count
         FROM clips
         LEFT JOIN collections ON collections.id = clips.collection_id
         LEFT JOIN images ON images.clip_id = clips.id
         LEFT JOIN file_clips ON file_clips.clip_id = clips.id
         {where_sql}
         ORDER BY clips.created_at DESC, clips.id DESC
         LIMIT ?"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let limit_param: Box<dyn rusqlite::ToSql> = Box::new(limit as i64);
    bind.push(limit_param);
    let bind_refs: Vec<&dyn rusqlite::ToSql> = bind.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();
    let mut rows = stmt.query(bind_refs.as_slice())?;

    let mut items: Vec<Clip> = Vec::new();
    let mut total: i64 = 0;
    let mut next_cursor: Option<String> = None;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        total = row.get::<_, i64>(24)?;
        let mut clip = decode_clip_row(&id, row)?;
        // Image meta (indices 18..22)
        let img_path: Option<String> = row.get(18)?;
        let img_thumb: Option<String> = row.get(19)?;
        let img_width: Option<i64> = row.get(20)?;
        let img_height: Option<i64> = row.get(21)?;
        let img_mime: Option<String> = row.get(22)?;
        if let (Some(path), Some(thumb_path), Some(width), Some(height), Some(mime)) =
            (img_path, img_thumb, img_width, img_height, img_mime)
        {
            clip.image = Some(ImageMeta {
                path,
                thumb_path,
                width,
                height,
                mime,
            });
        }
        // File paths (index 23)
        if let Some(paths_json) = row.get::<_, Option<String>>(23)? {
            let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
            if !paths.is_empty() {
                clip.file_paths = Some(paths);
            }
        }
        items.push(clip);
    }
    if items.len() == limit {
        if let Some(last) = items.last() {
            next_cursor = Some(last.created_at.to_string());
        }
    }
    Ok(SearchPage {
        items,
        total,
        next_cursor,
        took_ms: start.elapsed().as_millis() as u64,
    })
}

/// Cap the number of stored clips. Pinned + favorite clips are never deleted.
pub fn enforce_max_clips(conn: &DbConn, max_clips: i64) -> anyhow::Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
    if count <= max_clips {
        return Ok(0);
    }
    let to_delete = count - max_clips;
    let tx = conn.unchecked_transaction()?;
    // The clips_ad FTS trigger keeps clips_fts in sync.
    let affected = tx.execute(
        "DELETE FROM clips
         WHERE id IN (
           SELECT id FROM clips
           WHERE is_pinned = 0 AND is_favorite = 0
           ORDER BY created_at ASC
           LIMIT ?
         )",
        params![to_delete],
    )?;
    tx.commit()?;
    Ok(affected)
}

// ---------------------------------------------------------------------------
// Phase 6 / Fase 1: per-device multi-user support.
//
// The user table is intentionally minimal: an id, a display name, an optional
// email, an is_default flag (uniquely enforced by the partial index), and
// timestamps. We do NOT store passwords, passphrases, or anything sensitive
// in the user row — those live in the sync tables (Phase 6, cloud sync).
//
// We expose CRUD operations that all run in transactions where multi-step
// invariants matter (`set_default` clears the old default in the same tx).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// List every user in the DB, sorted by created_at ASC so the default user
/// is always at index 0 and newly-created users land at the bottom of the
/// dropdown. Cheap because we have at most a handful of users per device.
pub fn list_users(conn: &DbConn) -> anyhow::Result<Vec<User>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, display_name, email, is_default, created_at, updated_at
         FROM users ORDER BY created_at ASC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(User {
                id: r.get(0)?,
                display_name: r.get(1)?,
                email: r.get(2)?,
                is_default: r.get::<_, i64>(3)? != 0,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Look up a single user by id. Returns None if the id no longer exists
/// (e.g. it was deleted while another window had it cached).
pub fn get_user(conn: &DbConn, id: &str) -> anyhow::Result<Option<User>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, display_name, email, is_default, created_at, updated_at
         FROM users WHERE id = ?",
    )?;
    let user = stmt
        .query_row(params![id], |r| {
            Ok(User {
                id: r.get(0)?,
                display_name: r.get(1)?,
                email: r.get(2)?,
                is_default: r.get::<_, i64>(3)? != 0,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .ok();
    Ok(user)
}

/// Look up the current default user. We always have one (the unique partial
/// index guarantees it) but we still return Option for the same defensive
/// reasons as get_user: the unique index can theoretically be violated by
/// direct DB tampering, and we want a clean error path.
pub fn get_default_user(conn: &DbConn) -> anyhow::Result<Option<User>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, display_name, email, is_default, created_at, updated_at
         FROM users WHERE is_default = 1 LIMIT 1",
    )?;
    let user = stmt
        .query_row([], |r| {
            Ok(User {
                id: r.get(0)?,
                display_name: r.get(1)?,
                email: r.get(2)?,
                is_default: true,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .ok();
    Ok(user)
}

/// Create a new user. The id is a fresh ULID; the caller cannot supply one
/// (so we never get a collision). `email` is optional and not validated —
/// it's a label for the UI, not an auth identifier.
pub fn create_user(
    conn: &DbConn,
    display_name: &str,
    email: Option<&str>,
) -> anyhow::Result<User> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("display_name cannot be empty");
    }
    let id = Ulid::new().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO users (id, display_name, email, is_default, created_at, updated_at)
         VALUES (?, ?, ?, 0, ?, ?)",
        params![id, trimmed, email, now, now],
    )?;
    Ok(User {
        id,
        display_name: trimmed.to_string(),
        email: email.map(|s| s.to_string()),
        is_default: false,
        created_at: now,
        updated_at: now,
    })
}

/// Rename a user. We reject the rename of the default user's display name
/// to the empty string (the watcher and several UI affordances assume
/// there's always a non-empty default name to fall back on).
pub fn rename_user(conn: &DbConn, id: &str, display_name: &str) -> anyhow::Result<()> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("display_name cannot be empty");
    }
    let affected = conn.execute(
        "UPDATE users SET display_name = ?, updated_at = ? WHERE id = ?",
        params![trimmed, now_ms(), id],
    )?;
    if affected == 0 {
        anyhow::bail!("user {id} does not exist");
    }
    Ok(())
}

/// Set a user as the default. Runs in a transaction so the invariant "exactly
/// one user has is_default=1" is preserved even under racing writes. We
/// intentionally do not allow setting the default to the already-default
/// user (no-op write), but we DO allow switching it; the caller can keep
/// using whichever user the UI shows as default.
pub fn set_default_user(conn: &DbConn, id: &str) -> anyhow::Result<User> {
    let tx = conn.unchecked_transaction()?;
    // First make sure the target user actually exists.
    let user: User = {
        let mut stmt = tx.prepare_cached(
            "SELECT id, display_name, email, is_default, created_at, updated_at
             FROM users WHERE id = ?",
        )?;
        stmt.query_row(params![id], |r| {
            Ok(User {
                id: r.get(0)?,
                display_name: r.get(1)?,
                email: r.get(2)?,
                is_default: r.get::<_, i64>(3)? != 0,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .map_err(|_| anyhow::anyhow!("user {id} does not exist"))?
    };
    // The `stmt` is now out of scope; the borrow on `tx` is released so we
    // can run UPDATE statements on the same connection.
    let now = now_ms();
    tx.execute(
        "UPDATE users SET is_default = 0, updated_at = ? WHERE is_default = 1",
        params![now],
    )?;
    tx.execute(
        "UPDATE users SET is_default = 1, updated_at = ? WHERE id = ?",
        params![now, id],
    )?;
    tx.commit()?;
    Ok(User {
        is_default: true,
        updated_at: now,
        ..user
    })
}

/// Delete a user. Refuses to delete the default user (we always need a
/// default) and refuses to delete a user that still has clips referencing
/// it. The caller has to reassign or delete the user's clips first; the UI
/// surfaces that requirement with a clear error message.
pub fn delete_user(conn: &DbConn, id: &str) -> anyhow::Result<()> {
    // Refuse if it's the default.
    let is_default: i64 = conn
        .query_row(
            "SELECT is_default FROM users WHERE id = ?",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| anyhow::anyhow!("user {id} does not exist"))?;
    if is_default != 0 {
        anyhow::bail!("cannot delete the default user; reassign the default first");
    }
    // Refuse if any clip references it. We check `clips` because that is
    // the most user-facing table; the FKs from other tables cascade on
    // delete and the backfill would otherwise orphan image/ocr rows. If
    // you want to delete a non-default user, first reassign their clips
    // (a future bulk endpoint) or wipe the device.
    let clip_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM clips WHERE user_id = ?",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if clip_count > 0 {
        anyhow::bail!(
            "user has {clip_count} clip(s) — delete or reassign them first"
        );
    }
    conn.execute("DELETE FROM users WHERE id = ?", params![id])?;
    Ok(())
}

/// The user_id we hand to the watcher and to the list queries. We do not
/// cache this on AppState; the caller is expected to read the setting on
/// every call so a UI switch takes effect immediately.
///
/// Falls back to the default user if (a) the active_user_id setting is
/// missing, (b) it points at a user that no longer exists, or (c) the
/// default user itself is missing (which would be a schema violation,
/// but the fallback chain keeps the app from crashing).
pub fn resolve_active_user(conn: &DbConn, active_id: Option<&str>) -> anyhow::Result<String> {
    if let Some(id) = active_id {
        if get_user(conn, id)?.is_some() {
            return Ok(id.to_string());
        }
    }
    if let Some(u) = get_default_user(conn)? {
        return Ok(u.id);
    }
    // Last-ditch: the default user is missing, which means the DB is in
    // an inconsistent state. Recreate it and return its id. This is
    // best-effort — if it fails too, we bubble up the error.
    let id = "user_default".to_string();
    let now = now_ms();
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (id, display_name, is_default, created_at, updated_at)
         VALUES (?, 'Default', 1, ?, ?)",
        params![id, now, now],
    );
    Ok(id)
}
