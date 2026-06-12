import type { Clip } from "@/types";
import { relativeDateGroup } from "@/lib/utils";

/**
 * Pure function: groups clips into Today / Yesterday / Last Week / etc.
 * Exported separately so it can be unit tested without React.
 */
export function buildTimelineRows(clips: Clip[]) {
  const rows: Array<
    | { kind: "group"; label: string }
    | { kind: "clip"; clip: Clip }
  > = [];
  let lastGroup: string | null = null;
  for (const clip of clips) {
    const group = relativeDateGroup(clip.created_at);
    if (group !== lastGroup) {
      rows.push({ kind: "group", label: group });
      lastGroup = group;
    }
    rows.push({ kind: "clip", clip });
  }
  return rows;
}
