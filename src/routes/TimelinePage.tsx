import { useEffect, useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { FixedSizeList as List } from "react-window";
import { api } from "@/lib/tauri";
import { ClipRow } from "@/components/ClipRow";
import { Search } from "lucide-react";
import { useClips } from "@/features/clip/queries";
import { buildTimelineRows } from "@/features/timeline/grouping";

const ROW_HEIGHT = 64;

export function TimelinePage() {
  const [query, setQuery] = useState("");
  const [kindFilter, setKindFilter] = useState<string | null>(null);
  const [sourceFilter, setSourceFilter] = useState<string | null>(null);
  const [listHeight, setListHeight] = useState(() =>
    Math.max(200, window.innerHeight - 180)
  );

  const sourceApps = useQuery({
    queryKey: ["source-apps"],
    queryFn: api.listSourceApps,
  });

  const clips = useClips({
    query: query || undefined,
    kind: kindFilter ?? undefined,
    source_app: sourceFilter ?? undefined,
    limit: 500,
  });

  const rows = useMemo(() => buildTimelineRows(clips.data?.items ?? []), [clips.data]);

  useEffect(() => {
    const onResize = () => setListHeight(Math.max(200, window.innerHeight - 180));
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-3 border-b border-border px-6 py-3">
        <div className="relative flex-1 max-w-2xl">
          <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-fg-muted" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="input pl-9"
            placeholder="Search timeline..."
          />
        </div>
        <select
          value={kindFilter ?? ""}
          onChange={(e) => setKindFilter(e.target.value || null)}
          className="input max-w-[120px]"
        >
          <option value="">All types</option>
          <option value="text">Text</option>
          <option value="url">URL</option>
          <option value="image">Image</option>
          <option value="files">Files</option>
        </select>
        <select
          value={sourceFilter ?? ""}
          onChange={(e) => setSourceFilter(e.target.value || null)}
          className="input max-w-[180px]"
        >
          <option value="">All apps</option>
          {sourceApps.data?.map(([app, count]) => (
            <option key={app} value={app}>
              {app} ({count})
            </option>
          ))}
        </select>
      </header>
      <div className="flex-1 overflow-hidden px-6 py-4">
        {rows.length === 0 ? (
          <div className="empty-state">
            <Search className="h-8 w-8 opacity-30" />
            <p>No clips yet. Copy something to get started.</p>
          </div>
        ) : (
          <List
            itemCount={rows.length}
            itemSize={ROW_HEIGHT}
            height={listHeight}
            width="100%"
          >
            {({ index, style }) => {
              const row = rows[index];
              if (row.kind === "group") {
                return (
                  <div style={style} className="flex items-center px-2">
                    <h3 className="text-xs font-semibold uppercase tracking-wide text-fg-subtle">
                      {row.label}
                    </h3>
                    <div className="ml-3 h-px flex-1 bg-border" />
                  </div>
                );
              }
              return (
                <div style={style} className="group pr-2">
                  <ClipRow clip={row.clip} />
                </div>
              );
            }}
          </List>
        )}
      </div>
    </div>
  );
}
