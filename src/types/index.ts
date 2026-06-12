export type ClipType = "text" | "image" | "files" | "url";

export interface ImageMeta {
  path: string;
  thumb_path: string;
  width: number;
  height: number;
  mime: string;
}

export interface Clip {
  id: string;
  type: ClipType;
  content_hash: string;
  text_preview: string | null;
  byte_size: number;
  source_app: string | null;
  source_title: string | null;
  is_favorite: boolean;
  is_pinned: boolean;
  is_sensitive: boolean;
  collection_id: string | null;
  created_at: number;
  updated_at: number;
  usage_count: number;
  last_used_at: number | null;
  pinned_at: number | null;
  tags: string[];
  image: ImageMeta | null;
  file_paths: string[] | null;
  collection_name: string | null;
}

export interface SearchPage {
  items: Clip[];
  total: number;
  next_cursor: string | null;
  took_ms: number;
}

export interface Collection {
  id: string;
  name: string;
  icon: string | null;
  created_at: number;
  clip_count: number;
}

export interface Snippet {
  id: string;
  title: string;
  language: string;
  body: string;
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
}

export type ThemeMode = "system" | "light" | "dark" | "graphite";

export interface Settings {
  retention_days: number;
  max_clips: number;
  hotkey: string;
  theme: ThemeMode;
  autostart: boolean;
  storage_dir: string | null;
  excluded_apps: string[];
  sensitive_apps: string[];
  auto_paste: boolean;
  backup_enabled: boolean;
  backup_dir: string | null;
  /**
   * When true (the default), ClipVault does not perform any network
   * requests. `sync_endpoint` and `http_receiver_enabled` are ignored.
   */
  local_only: boolean;
  sync_endpoint: string | null;
  http_receiver_enabled: boolean;
  ring_hotkey_reverse: string;
  ring_hotkey_forward: string;
  ring_hotkey_overlay: string;
  ring_capacity: number;
  ring_idle_dismiss_ms: number;
  ring_wrap: boolean;
  ring_include_sensitive: boolean;
  ring_include_files: boolean;
  ring_include_images: boolean;
  /** String inserted between clips when the user merges a multi-selection. */
  merge_separator: string;
  /** Rows to jump when the user holds Ctrl while pressing ↑/↓ in the palette. 0 = top/bottom. */
  palette_jump_size: number;
}
