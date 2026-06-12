import { z } from "zod";

/**
 * Zod schemas for forms. Centralized so the Settings page, collection create
 * form, snippet editor, and command palette input all share the same rules.
 */

export const hotkeySchema = z
  .string()
  .min(3)
  .max(64)
  .regex(
    /^((Ctrl|Alt|Shift|Meta|CommandOrControl)\+)*[A-Z0-9]([A-Z0-9]?)$/i,
    "Use modifiers like Ctrl/Shift/Alt + a single key (e.g. Ctrl+Shift+V)"
  );

export const settingsSchema = z.object({
  retention_days: z.number().int().min(0).max(36500),
  max_clips: z.number().int().min(1_000).max(100_000_000),
  hotkey: hotkeySchema,
  theme: z.enum(["system", "light", "dark", "graphite"]),
  autostart: z.boolean(),
  storage_dir: z.string().nullable(),
  excluded_apps: z.array(z.string().min(1)).max(500),
  sensitive_apps: z.array(z.string().min(1)).max(500),
  auto_paste: z.boolean(),
  backup_enabled: z.boolean(),
  backup_dir: z.string().nullable(),
  local_only: z.boolean(),
  sync_endpoint: z.string().url().nullable().or(z.literal("")).transform((v) => (v ? v : null)),
  http_receiver_enabled: z.boolean(),
  // Clipboard Ring
  ring_hotkey_reverse: hotkeySchema,
  ring_hotkey_forward: hotkeySchema,
  ring_hotkey_overlay: hotkeySchema,
  ring_capacity: z.number().int().min(1).max(1024),
  ring_idle_dismiss_ms: z.number().int().min(0).max(60 * 60 * 1000),
  ring_wrap: z.boolean(),
  ring_include_sensitive: z.boolean(),
  ring_include_files: z.boolean(),
  ring_include_images: z.boolean(),
});

export type SettingsInput = z.infer<typeof settingsSchema>;

export const collectionSchema = z.object({
  name: z
    .string()
    .min(1, "Name is required")
    .max(64, "Name must be 64 characters or fewer")
    .regex(/^[^/\\<>:"|?*]+$/, "Name contains invalid characters"),
  icon: z.string().max(8).nullable(),
});

export const snippetSchema = z.object({
  title: z.string().min(1, "Title is required").max(120),
  language: z.string().min(1).max(32),
  body: z.string().min(1, "Snippet body is empty"),
  is_favorite: z.boolean(),
});
