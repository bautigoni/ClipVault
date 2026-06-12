import { describe, expect, it } from "vitest";
import { collectionSchema, hotkeySchema, settingsSchema, snippetSchema } from "./schemas";

const fullSettings = {
  retention_days: 0,
  max_clips: 1_000_000,
  hotkey: "Ctrl+Shift+V",
  theme: "system",
  autostart: false,
  storage_dir: null,
  excluded_apps: [],
  sensitive_apps: [],
  auto_paste: true,
  backup_enabled: false,
  backup_dir: null,
  local_only: true,
  sync_endpoint: null,
  http_receiver_enabled: false,
  ring_hotkey_reverse: "Ctrl+Shift+V",
  ring_hotkey_forward: "Ctrl+Shift+Alt+V",
  ring_hotkey_overlay: "Ctrl+Shift+R",
  ring_capacity: 64,
  ring_idle_dismiss_ms: 30_000,
  ring_wrap: true,
  ring_include_sensitive: false,
  ring_include_files: true,
  ring_include_images: true,
};

describe("settingsSchema", () => {
  it("accepts the default settings", () => {
    const result = settingsSchema.safeParse(fullSettings);
    expect(result.success).toBe(true);
  });

  it("rejects negative retention", () => {
    const result = settingsSchema.safeParse({ ...fullSettings, retention_days: -1 });
    expect(result.success).toBe(false);
  });

  it("rejects an invalid hotkey", () => {
    const result = settingsSchema.safeParse({ ...fullSettings, hotkey: "potato" });
    expect(result.success).toBe(false);
  });

  it("rejects ring capacity out of range", () => {
    expect(settingsSchema.safeParse({ ...fullSettings, ring_capacity: 0 }).success).toBe(false);
    expect(settingsSchema.safeParse({ ...fullSettings, ring_capacity: 9999 }).success).toBe(false);
  });

  it("rejects non-boolean local_only", () => {
    expect(settingsSchema.safeParse({ ...fullSettings, local_only: "yes" }).success).toBe(false);
  });

  it("accepts local_only: false (cloud mode opt-in)", () => {
    expect(settingsSchema.safeParse({ ...fullSettings, local_only: false }).success).toBe(true);
  });
});

describe("collectionSchema", () => {
  it("requires a non-empty name", () => {
    expect(collectionSchema.safeParse({ name: "", icon: null }).success).toBe(false);
    expect(collectionSchema.safeParse({ name: "Servers", icon: null }).success).toBe(true);
  });
  it("rejects filesystem-unsafe characters", () => {
    expect(collectionSchema.safeParse({ name: "foo/bar", icon: null }).success).toBe(false);
  });
});

describe("snippetSchema", () => {
  it("requires a body", () => {
    expect(
      snippetSchema.safeParse({ title: "x", language: "sql", body: "", is_favorite: false })
        .success
    ).toBe(false);
  });
});

describe("hotkeySchema", () => {
  it.each([
    ["Ctrl+Shift+V", true],
    ["Alt+F4", true],
    ["Ctrl+1", true],
    ["potato", false],
    ["Ctrl+", false],
  ])("hotkey %s -> valid=%s", (input, valid) => {
    expect(hotkeySchema.safeParse(input).success).toBe(valid);
  });
});
