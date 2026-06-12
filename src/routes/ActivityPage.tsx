import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Trash2, Activity as ActivityIcon } from "lucide-react";
import { useMemo } from "react";
import { api, type ActivityEntry } from "@/lib/tauri";

/**
 * Activity log page.
 *
 * Renders the most recent 200 activity entries from the backend. Entries are
 * metadata-only — we never log clipboard *content* — so this view is safe
 * to share with a developer for debugging without leaking secrets.
 */
export function ActivityPage() {
  const qc = useQueryClient();
  const { data, isLoading } = useQuery({
    queryKey: ["activity"],
    queryFn: () => api.listActivity(500),
    refetchInterval: 3000,
  });

  const grouped = useMemo(() => groupByDay(data ?? []), [data]);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-2">
          <ActivityIcon className="h-5 w-5 text-accent" />
          <h2 className="text-lg font-semibold text-fg">Activity</h2>
          <span className="text-xs text-fg-muted">
            {data ? `${data.length} event${data.length === 1 ? "" : "s"}` : ""}
          </span>
        </div>
        <button
          type="button"
          onClick={async () => {
            if (!confirm("Clear all activity log entries? (No clips will be affected.)")) return;
            await api.clearActivity();
            qc.invalidateQueries({ queryKey: ["activity"] });
          }}
          className="flex items-center gap-1.5 rounded-md border border-border bg-bg px-3 py-1.5 text-xs text-fg-muted hover:bg-bg-overlay hover:text-fg"
        >
          <Trash2 className="h-3.5 w-3.5" />
          Clear log
        </button>
      </header>

      <div className="flex-1 overflow-auto p-6">
        {isLoading && (
          <p className="text-sm text-fg-muted">Loading activity…</p>
        )}
        {!isLoading && (data?.length ?? 0) === 0 && (
          <div className="flex h-full flex-col items-center justify-center text-center">
            <ActivityIcon className="h-10 w-10 text-fg-subtle" />
            <p className="mt-3 text-sm font-medium text-fg">No activity yet</p>
            <p className="mt-1 text-xs text-fg-muted">
              Copy something into ClipVault — every clip you create, paste,
              or transform will show up here.
            </p>
          </div>
        )}
        {grouped.map(([day, entries]) => (
          <section key={day} className="mb-6">
            <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-fg-muted">
              {day}
            </h3>
            <ul className="space-y-1">
              {entries.map((e) => (
                <ActivityRow key={e.id} entry={e} />
              ))}
            </ul>
          </section>
        ))}
      </div>
    </div>
  );
}

function ActivityRow({ entry }: { entry: ActivityEntry }) {
  const ts = new Date(entry.ts_ms);
  const time = ts.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  return (
    <li className="grid grid-cols-[80px_140px_1fr] items-baseline gap-3 rounded-md border border-border bg-bg-elevated px-3 py-2 text-xs">
      <span className="font-mono text-fg-muted">{time}</span>
      <span className="truncate font-mono text-accent">{entry.kind}</span>
      <span className="truncate text-fg-muted">
        {entry.source_app && <span className="mr-2 rounded bg-bg-overlay px-1.5 py-0.5">{entry.source_app}</span>}
        {entry.detail}
        {entry.clip_id && (
          <span className="ml-2 font-mono text-fg-subtle">· {entry.clip_id.slice(0, 8)}</span>
        )}
      </span>
    </li>
  );
}

function groupByDay(entries: ActivityEntry[]): Array<[string, ActivityEntry[]]> {
  const out: Array<[string, ActivityEntry[]]> = [];
  for (const e of entries) {
    const d = new Date(e.ts_ms);
    const day = d.toLocaleDateString([], { year: "numeric", month: "short", day: "numeric" });
    if (out.length === 0 || out[out.length - 1][0] !== day) {
      out.push([day, [e]]);
    } else {
      out[out.length - 1][1].push(e);
    }
  }
  return out;
}
