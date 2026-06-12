import { invoke } from "@tauri-apps/api/core";
import type { Clip, Collection, SearchPage, Settings, Snippet } from "@/types";

export interface ClipFilter {
  query?: string;
  limit?: number;
  cursor?: string;
  source_app?: string;
  collection_id?: string;
  kind?: string;
  favorites_only?: boolean;
  pinned_only?: boolean;
  tag?: string;
}

export const api = {
  searchClips: (filter: ClipFilter) =>
    invoke<SearchPage>("search_clips", {
      query: filter.query ?? null,
      limit: filter.limit ?? 50,
      cursor: filter.cursor ?? null,
      sourceApp: filter.source_app ?? null,
      collectionId: filter.collection_id ?? null,
      kind: filter.kind ?? null,
      favoritesOnly: filter.favorites_only ?? false,
      pinnedOnly: filter.pinned_only ?? false,
      tag: filter.tag ?? null,
    }),
  listClips: (filter: ClipFilter) =>
    invoke<SearchPage>("list_clips", {
      // `list_clips` in Rust does not accept a `query` argument; we strip it
      // here so extra fields are not silently dropped on the way down.
      limit: filter.limit ?? 100,
      cursor: filter.cursor ?? null,
      sourceApp: filter.source_app ?? null,
      collectionId: filter.collection_id ?? null,
      kind: filter.kind ?? null,
      favoritesOnly: filter.favorites_only ?? false,
      pinnedOnly: filter.pinned_only ?? false,
      tag: filter.tag ?? null,
    }),
  getClip: (id: string) => invoke<Clip | null>("get_clip", { id }),

  toggleFavorite: (id: string, value: boolean) =>
    invoke<void>("toggle_favorite", { id, value }),
  pinClip: (id: string, pinned: boolean) => invoke<void>("pin_clip", { id, pinned }),
  deleteClip: (id: string) => invoke<void>("delete_clip", { id }),
  deleteClips: (ids: string[]) => invoke<number>("delete_clips", { ids }),
  clearHistory: () => invoke<number>("clear_history"),
  updateClipMeta: (id: string, patch: Record<string, unknown>) =>
    invoke<Clip>("update_clip_meta", { id, patch }),

  listCollections: () => invoke<Collection[]>("list_collections"),
  createCollection: (name: string, icon: string | null) =>
    invoke<Collection>("create_collection", { name, icon }),
  deleteCollection: (id: string) => invoke<void>("delete_collection", { id }),
  renameCollection: (id: string, name: string, icon: string | null) =>
    invoke<void>("rename_collection", { id, name, icon }),
  assignToCollection: (clipId: string, collectionId: string | null) =>
    invoke<void>("assign_to_collection", { clipId, collectionId }),

  listSnippets: () => invoke<Snippet[]>("list_snippets"),
  searchSnippets: (query: string, limit = 100) =>
    invoke<Snippet[]>("search_snippets", { query, limit }),
  upsertSnippet: (input: {
    id?: string;
    title: string;
    language: string;
    body: string;
    is_favorite: boolean;
  }) => invoke<Snippet>("upsert_snippet", { input }),
  deleteSnippet: (id: string) => invoke<void>("delete_snippet", { id }),
  copySnippetToClipboard: (id: string) => invoke<void>("copy_snippet_to_clipboard", { id }),

  readImageFull: (relPath: string) => invoke<number[]>("read_image_full", { relPath }),
  readImageThumb: (relPath: string) => invoke<number[]>("read_image_thumb", { relPath }),

  exportDb: (path: string) => invoke<void>("export_db", { req: { path } }),
  importDb: (path: string, policy: "skip" | "overwrite" | "duplicate") =>
    invoke<{
      clips_added: number;
      clips_skipped: number;
      collections_added: number;
      snippets_added: number;
      errors: string[];
    }>("import_db", { req: { path, policy } }),

  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (patch: Partial<Settings>) => invoke<Settings>("update_settings", { patch }),

  showPalette: () => invoke<void>("show_palette"),
  hidePalette: () => invoke<void>("hide_palette"),
  showMain: () => invoke<void>("show_main"),
  hideMain: () => invoke<void>("hide_main"),
  registerHotkey: (combo: string) => invoke<void>("register_hotkey", { combo }),

  copyClipToClipboard: (id: string) => invoke<void>("copy_clip_to_clipboard", { id }),
  mergeAndPasteClips: (ids: string[]) => invoke<string>("merge_and_paste_clips", { ids }),
  listSourceApps: () => invoke<[string, number][]>("list_source_apps"),
  listTags: () => invoke<string[]>("list_tags"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  runBackup: () => invoke<string>("run_backup"),

  // Clipboard Ring
  ringSetScope: (
    scope: {
      kind: "global" | "favorites" | "collection" | "application" | "kind" | "named_set";
      collection_id?: string;
      application_exe?: string;
      clip_kind?: string;
      set_id?: string;
    },
    config?: {
      capacity?: number;
      wrap?: boolean;
      idle_dismiss_ms?: number;
      include_sensitive?: boolean;
      include_files?: boolean;
      include_images?: boolean;
    },
  ) => invoke<number>("ring_set_scope", { scope, config: config ?? null }),
  ringReverse: () => invoke<RingActionResult>("ring_reverse"),
  ringForward: () => invoke<RingActionResult>("ring_forward"),
  ringJump: (index: number) => invoke<RingActionResult>("ring_jump", { index }),
  ringDismiss: () => invoke<void>("ring_dismiss"),
  ringIsActive: () => invoke<boolean>("ring_is_active"),
  ringPreview: (n = 5) => invoke<RingSlotView[]>("ring_preview", { n }),
  ringListSets: () => invoke<RingSet[]>("ring_list_sets"),
  ringCreateSet: (name: string, scope_kind: string, scope_ref: string | null) =>
    invoke<RingSet>("ring_create_set", { name, scopeKind: scope_kind, scopeRef: scope_ref }),
  ringDeleteSet: (id: string) => invoke<void>("ring_delete_set", { id }),
  ringAddToSet: (setId: string, clipId: string, position?: number) =>
    invoke<void>("ring_add_to_set", { setId, clipId, position: position ?? null }),
  ringRemoveFromSet: (setId: string, clipId: string) =>
    invoke<void>("ring_remove_from_set", { setId, clipId }),

  // Diagnostic: triggers the auto-paste SendInput sequence without touching
  // the clipboard. See debug.log for what happened.
  testPaste: () => invoke<void>("test_paste"),
};

export interface RingSlotView {
  index: number;
  total: number;
  clip_id: string;
  preview: string;
  kind: string;
  is_pinned: boolean;
  is_favorite: boolean;
}

export interface RingSet {
  id: string;
  name: string;
  scope_kind: string;
  scope_ref: string | null;
  created_at: number;
  updated_at: number;
  item_count: number;
}

export type RingActionResult =
  | { kind: "activated"; id: string; index: number; total: number }
  | { kind: "no_op" }
  | { kind: "pruned" }
  | { kind: "empty" }
  | { kind: "skipped" }
  | { kind: "failed"; error: string };
