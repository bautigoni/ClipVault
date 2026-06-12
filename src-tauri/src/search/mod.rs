//! Search engine: FTS5 first, with a fuzzy fallback for typo tolerance.

use rusqlite::params;
use strsim::levenshtein;

use crate::db::repo::ClipFilter;
use crate::db::DbPool;
use crate::state::SearchPage;

/// Search the clip history. If FTS returns nothing (or the query looks noisy), apply
/// a Levenshtein-based fuzzy fallback over the recent clip previews.
pub fn search(pool: &DbPool, filter: &ClipFilter, limit: usize, cursor: Option<&str>) -> anyhow::Result<SearchPage> {
    // Try the structured path first.
    let page = crate::db::repo::search_clips(pool, filter, limit, cursor)?;
    if !page.items.is_empty() {
        return Ok(page);
    }
    if let Some(q) = filter.query.as_ref().filter(|q| !q.trim().is_empty()) {
        if let Ok(fuzzy) = fuzzy_fallback(pool, q, limit) {
            if !fuzzy.items.is_empty() {
                return Ok(fuzzy);
            }
        }
    }
    Ok(page)
}

const FUZZY_MAX_DISTANCE_RATIO: f64 = 0.4;
const FUZZY_CANDIDATE_LIMIT: i64 = 2000;

fn fuzzy_fallback(pool: &DbPool, query: &str, limit: usize) -> anyhow::Result<SearchPage> {
    let start = std::time::Instant::now();
    let conn = pool.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT id, text_preview, source_app, source_title FROM clips
         WHERE text_preview IS NOT NULL
         ORDER BY created_at DESC LIMIT ?",
    )?;
    let query_norm = normalize(query);
    let tokens: Vec<&str> = query_norm.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(SearchPage::default());
    }
    let max_distance = ((query_norm.chars().count() as f64) * FUZZY_MAX_DISTANCE_RATIO).ceil() as usize;
    let mut scored: Vec<(i64, String)> = Vec::new();
    let mut rows = stmt.query(params![FUZZY_CANDIDATE_LIMIT])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let preview: String = row.get(1)?;
        let preview_norm = normalize(&preview);
        let mut best_token_distance = usize::MAX;
        for token in tokens.iter() {
            let distance = best_token_distance_for(token, &preview_norm);
            if distance < best_token_distance {
                best_token_distance = distance;
            }
        }
        if best_token_distance <= max_distance {
            // Lower distance == better. Convert to a positive score (higher is better).
            let score = (max_distance as i64) - (best_token_distance as i64);
            scored.push((score, id));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(limit);
    if scored.is_empty() {
        return Ok(SearchPage::default());
    }
    let placeholders = std::iter::repeat("?").take(scored.len()).collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT clips.id, clips.type, clips.content_hash, clips.text_preview, clips.byte_size,
                clips.source_app, clips.source_title, clips.is_favorite, clips.is_pinned,
                clips.is_sensitive, clips.collection_id, clips.created_at, clips.updated_at,
                clips.usage_count, clips.last_used_at, clips.pinned_at, collections.name
         FROM clips
         LEFT JOIN collections ON collections.id = clips.collection_id
         WHERE clips.id IN ({placeholders})"
    );
    let mut stmt = conn.prepare(&sql)?;
    let bound_refs: Vec<&dyn rusqlite::ToSql> = scored.iter().map(|(_, id)| id as &dyn rusqlite::ToSql).collect();
    let mut rows = stmt.query(bound_refs.as_slice())?;
    let mut items = Vec::new();
    while let Some(row) = rows.next()? {
        let clip = crate::db::repo::row_to_clip(&*conn, row, true)?;
        items.push(clip);
    }
    Ok(SearchPage {
        items,
        total: scored.len() as i64,
        next_cursor: None,
        took_ms: start.elapsed().as_millis() as u64,
    })
}

fn best_token_distance_for(token: &str, haystack: &str) -> usize {
    let mut best = usize::MAX;
    for word in haystack.split_whitespace() {
        if word.is_empty() {
            continue;
        }
        let d = levenshtein(token, word);
        if d < best {
            best = d;
        }
        // Cheap prefix boost: if the word starts with the token, treat as exact.
        if word.starts_with(token) {
            return 0;
        }
    }
    best
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
