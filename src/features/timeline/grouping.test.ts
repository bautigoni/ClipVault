import { describe, expect, it } from "vitest";
import { buildTimelineRows } from "./grouping";
import type { Clip } from "@/types";

function makeClip(createdAt: number, id = `c${createdAt}`): Clip {
  return {
    id,
    type: "text",
    content_hash: id,
    text_preview: id,
    byte_size: id.length,
    source_app: "test.exe",
    source_title: null,
    is_favorite: false,
    is_pinned: false,
    is_sensitive: false,
    collection_id: null,
    created_at: createdAt,
    updated_at: createdAt,
    usage_count: 1,
    last_used_at: null,
    pinned_at: null,
    tags: [],
    image: null,
    file_paths: null,
    collection_name: null,
  };
}

describe("buildTimelineRows", () => {
  it("returns an empty array for no clips", () => {
    expect(buildTimelineRows([])).toEqual([]);
  });

  it("inserts a group header when the group changes", () => {
    const now = Date.now();
    const clips = [
      makeClip(now, "a"),
      makeClip(now - 86_400_000, "b"),
      makeClip(now - 7 * 86_400_000, "c"),
    ];
    const rows = buildTimelineRows(clips);
    const groups = rows.filter((r) => r.kind === "group");
    expect(groups.length).toBe(3);
    expect(groups[0]).toMatchObject({ kind: "group", label: "Today" });
  });

  it("deduplicates consecutive group headers", () => {
    const now = Date.now();
    const clips = [
      makeClip(now, "a"),
      makeClip(now - 60_000, "b"),
      makeClip(now - 5 * 60_000, "c"),
    ];
    const rows = buildTimelineRows(clips);
    const groups = rows.filter((r) => r.kind === "group");
    expect(groups.length).toBe(1);
    // Three clip rows + one group row
    expect(rows.length).toBe(4);
  });

  it("preserves the original clip order", () => {
    const now = Date.now();
    const clips = [
      makeClip(now, "a"),
      makeClip(now - 86_400_000, "b"),
      makeClip(now - 86_400_000, "c"),
    ];
    const rows = buildTimelineRows(clips);
    const clipRows = rows.filter((r) => r.kind === "clip") as Array<{ kind: "clip"; clip: Clip }>;
    expect(clipRows.map((r) => r.clip.id)).toEqual(["a", "b", "c"]);
  });
});
