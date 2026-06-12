//! Ring buffer data structure: a logical view over the recent clips table.

use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// A clip identifier (ULID string). Re-exported as a type alias for readability.
pub type ClipId = String;
/// A collection identifier.
pub type CollectionId = String;
/// A ring-set identifier.
pub type RingSetId = String;
/// A clip kind filter.
pub type ClipType = String;

/// The scope the ring is currently looking at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RingScope {
    /// Every recorded clip, most recent first.
    Global,
    /// Only `is_favorite = 1` clips.
    Favorites,
    /// Only clips where `collection_id = X`.
    Collection(CollectionId),
    /// Only clips where `source_app = X`.
    Application { exe: String },
    /// Only clips of a given `type` (text, url, image, files).
    Kind(ClipType),
    /// An ad-hoc, named set the user builds.
    NamedSet(RingSetId),
}

impl Default for RingScope {
    fn default() -> Self {
        RingScope::Global
    }
}

impl RingScope {
    /// Stable string used as a cache key when rebuilding the buffer.
    pub fn cache_key(&self) -> String {
        match self {
            RingScope::Global => "global".into(),
            RingScope::Favorites => "favorites".into(),
            RingScope::Collection(id) => format!("collection:{id}"),
            RingScope::Application { exe } => format!("app:{exe}"),
            RingScope::Kind(k) => format!("kind:{k}"),
            RingScope::NamedSet(id) => format!("set:{id}"),
        }
    }
}

/// Per-scope configuration for the ring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingConfig {
    /// Max number of slots. Most recent first; older entries are dropped on rebuild.
    pub capacity: usize,
    /// Whether cycling past the edges wraps around.
    pub wrap: bool,
    /// Idle dismiss timeout in milliseconds; 0 disables.
    pub idle_dismiss_ms: u64,
    /// Whether to include clips flagged as sensitive.
    pub include_sensitive: bool,
    /// Whether to include file clips.
    pub include_files: bool,
    /// Whether to include image clips.
    pub include_images: bool,
}

impl Default for RingConfig {
    fn default() -> Self {
        Self {
            capacity: 64,
            wrap: true,
            idle_dismiss_ms: 30_000,
            include_sensitive: false,
            include_files: true,
            include_images: true,
        }
    }
}

/// A lightweight view of a ring slot for the optional overlay UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSlotView {
    pub index: usize,
    pub total: usize,
    pub clip_id: ClipId,
    pub preview: String,
    pub kind: ClipType,
    pub is_pinned: bool,
    pub is_favorite: bool,
}

/// The in-memory ring. Most recent first.
///
/// The ring is a *view* over the persistent `clips` table; the slot list is
/// refreshed on demand (cold start, scope change, large time gap, watcher
/// invalidation) by calling [`RingBuffer::rebuild`].
pub struct RingBuffer {
    pub scope: RingScope,
    pub config: RingConfig,
    /// Most recent first. Length <= config.capacity.
    slots: VecDeque<ClipId>,
    /// Index into `slots` of the *currently active* slot, or None if dismissed.
    cursor: Option<usize>,
    /// When the user last interacted; used for idle dismiss.
    last_touch: Option<Instant>,
    /// What type each slot is, cached for the overlay preview.
    kinds: Vec<ClipType>,
    previews: Vec<Option<String>>,
    pinned: Vec<bool>,
    favorites: Vec<bool>,
    /// When `true`, the slot list may be stale and the controller should call
    /// `rebuild` before the next user interaction. Set by `invalidate` and
    /// checked by `is_dirty`.
    dirty: bool,
}

impl RingBuffer {
    pub fn new(scope: RingScope, config: RingConfig) -> Self {
        Self {
            scope,
            config,
            slots: VecDeque::new(),
            cursor: None,
            last_touch: None,
            kinds: Vec::new(),
            previews: Vec::new(),
            pinned: Vec::new(),
            favorites: Vec::new(),
            dirty: true,
        }
    }

    /// Replace the slot list. Resets the cursor to `None` (dismissed).
    pub fn rebuild(
        &mut self,
        items: Vec<SlotData>,
    ) {
        let cap = self.config.capacity.max(1);
        self.slots.clear();
        self.kinds.clear();
        self.previews.clear();
        self.pinned.clear();
        self.favorites.clear();
        for item in items.into_iter().take(cap) {
            self.slots.push_back(item.id);
            self.kinds.push(item.kind);
            self.previews.push(item.preview);
            self.pinned.push(item.is_pinned);
            self.favorites.push(item.is_favorite);
        }
        self.cursor = None;
        self.last_touch = None;
        self.dirty = false;
    }

    /// Mark the slot list as possibly stale. The next access that needs an
    /// up-to-date view should call `rebuild`.
    pub fn invalidate(&mut self) {
        self.dirty = true;
        self.dismiss();
    }

    /// Returns true if the slot list needs to be rebuilt before being used.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Move the cursor to the *older* slot (towards the tail) and return its id.
    /// First press of a fresh session starts at the *oldest* visible slot, matching
    /// the user-visible behavior in the design brief.
    pub fn reverse(&mut self) -> Option<&ClipId> {
        if self.slots.is_empty() {
            return None;
        }
        let next = match self.cursor {
            None => self.slots.len() - 1,
            Some(0) if self.config.wrap => self.slots.len() - 1,
            Some(0) => return self.active(),
            Some(i) => i - 1,
        };
        self.cursor = Some(next);
        self.last_touch = Some(Instant::now());
        self.slots.get(next)
    }

    /// Move the cursor to the *newer* slot (towards the head).
    pub fn forward(&mut self) -> Option<&ClipId> {
        if self.slots.is_empty() {
            return None;
        }
        let next = match self.cursor {
            None => 0,
            Some(i) => {
                let max = self.slots.len() - 1;
                if i >= max {
                    if self.config.wrap { 0 } else { return self.active(); }
                } else {
                    i + 1
                }
            }
        };
        self.cursor = Some(next);
        self.last_touch = Some(Instant::now());
        self.slots.get(next)
    }

    /// Jump to a specific slot by index. Returns None if the index is out of range.
    pub fn jump(&mut self, slot: usize) -> Option<&ClipId> {
        if slot >= self.slots.len() {
            return None;
        }
        self.cursor = Some(slot);
        self.last_touch = Some(Instant::now());
        self.slots.get(slot)
    }

    /// Dismiss the ring (cursor = None). The next press re-initializes the cursor.
    pub fn dismiss(&mut self) {
        self.cursor = None;
        self.last_touch = None;
    }

    pub fn is_active(&self) -> bool {
        self.cursor.is_some()
    }

    pub fn active(&self) -> Option<&ClipId> {
        self.cursor.and_then(|i| self.slots.get(i))
    }

    pub fn active_index(&self) -> Option<usize> {
        self.cursor
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Returns true if the idle timeout has elapsed since the last user interaction.
    pub fn is_idle_timed_out(&self) -> bool {
        if self.config.idle_dismiss_ms == 0 || self.last_touch.is_none() {
            return false;
        }
        let elapsed_ms = self
            .last_touch
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);
        elapsed_ms >= self.config.idle_dismiss_ms
    }

    /// Build a preview for the optional overlay UI (next `n` slots starting at the cursor).
    pub fn preview(&self, n: usize) -> Vec<RingSlotView> {
        if self.slots.is_empty() {
            return Vec::new();
        }
        let start = self.cursor.unwrap_or(0);
        let total = self.slots.len();
        (0..n.min(total))
            .map(|i| {
                let idx = (start + i) % total;
                RingSlotView {
                    index: idx,
                    total,
                    clip_id: self.slots[idx].clone(),
                    preview: self.previews.get(idx).cloned().flatten().unwrap_or_default(),
                    kind: self.kinds.get(idx).cloned().unwrap_or_else(|| "text".into()),
                    is_pinned: self.pinned.get(idx).copied().unwrap_or(false),
                    is_favorite: self.favorites.get(idx).copied().unwrap_or(false),
                }
            })
            .collect()
    }
}

/// What the controller needs from a `clips` row to populate the buffer.
#[derive(Debug, Clone)]
pub struct SlotData {
    pub id: ClipId,
    pub kind: ClipType,
    pub preview: Option<String>,
    pub is_pinned: bool,
    pub is_favorite: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buffer(n: usize) -> RingBuffer {
        let mut buf = RingBuffer::new(RingScope::Global, RingConfig::default());
        let items: Vec<SlotData> = (0..n)
            .map(|i| SlotData {
                id: format!("c{i}"),
                kind: "text".into(),
                preview: Some(format!("item {i}")),
                is_pinned: false,
                is_favorite: false,
            })
            .collect();
        buf.rebuild(items);
        buf
    }

    #[test]
    fn first_reverse_starts_at_oldest() {
        let buf = make_buffer(3);
        assert_eq!(buf.reverse().unwrap(), "c2");
    }

    #[test]
    fn reverse_then_reverse_keeps_aging() {
        let mut buf = make_buffer(3);
        assert_eq!(buf.reverse().unwrap(), "c2");
        assert_eq!(buf.reverse().unwrap(), "c1");
        assert_eq!(buf.reverse().unwrap(), "c0");
    }

    #[test]
    fn reverse_with_wrap_loops_back() {
        let mut buf = make_buffer(3);
        for _ in 0..3 {
            buf.reverse();
        }
        assert_eq!(buf.reverse().unwrap(), "c2");
    }

    #[test]
    fn reverse_without_wrap_stops_at_head() {
        let mut buf = RingBuffer::new(RingScope::Global, RingConfig {
            wrap: false,
            ..RingConfig::default()
        });
        buf.rebuild((0..3).map(|i| SlotData {
            id: format!("c{i}"),
            kind: "text".into(),
            preview: None,
            is_pinned: false,
            is_favorite: false,
        }).collect());
        buf.reverse();
        buf.reverse();
        buf.reverse();
        // After three presses at the head, cursor stays at the head
        assert_eq!(buf.reverse().unwrap(), "c0");
    }

    #[test]
    fn forward_advances_toward_head() {
        let mut buf = make_buffer(3);
        assert_eq!(buf.forward().unwrap(), "c0");
        assert_eq!(buf.forward().unwrap(), "c1");
    }

    #[test]
    fn dismiss_resets_cursor() {
        let mut buf = make_buffer(3);
        buf.reverse();
        assert!(buf.is_active());
        buf.dismiss();
        assert!(!buf.is_active());
        // Next reverse starts fresh at the oldest.
        assert_eq!(buf.reverse().unwrap(), "c2");
    }

    #[test]
    fn empty_buffer_returns_none() {
        let mut buf = RingBuffer::new(RingScope::Global, RingConfig::default());
        assert!(buf.reverse().is_none());
        assert!(buf.forward().is_none());
        assert!(!buf.is_active());
    }

    #[test]
    fn jump_clips_index() {
        let mut buf = make_buffer(5);
        assert_eq!(buf.jump(2).unwrap(), "c2");
        assert_eq!(buf.active().unwrap(), "c2");
    }

    #[test]
    fn jump_out_of_range_returns_none() {
        let mut buf = make_buffer(3);
        assert!(buf.jump(99).is_none());
    }

    #[test]
    fn preview_starts_at_cursor() {
        let mut buf = make_buffer(5);
        buf.reverse();
        buf.reverse();
        // cursor at index 3, preview starts there
        let view = buf.preview(3);
        assert_eq!(view[0].clip_id, "c3");
        assert_eq!(view.len(), 3);
    }
}
