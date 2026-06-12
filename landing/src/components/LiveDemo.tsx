import { useEffect, useMemo, useState } from "react";
import { Search, Combine, ArrowDown, ArrowUp, CornerDownLeft } from "lucide-react";

type Sample = { id: string; app: string; type: "text" | "url" | "image"; text: string };

const SAMPLES: Sample[] = [
  { id: "1", app: "Chrome", type: "url", text: "https://github.com/tauri-apps/tauri" },
  { id: "2", app: "VSCode", type: "text", text: "useEffect(() => { /* mount */ }, []);" },
  { id: "3", app: "Slack", type: "text", text: "deploy logs from staging — all green" },
  { id: "4", app: "Cursor", type: "text", text: "SELECT id, hash FROM clips WHERE id = ?" },
  { id: "5", app: "Notepad", type: "text", text: "shopping list: milk, eggs, coffee" },
  { id: "6", app: "GitHub Desktop", type: "url", text: "https://github.com/bautigoni/ClipVault" },
  { id: "7", app: "Claude", type: "text", text: "shortcut to remember: Ctrl + Shift + V" },
  { id: "8", app: "Obsidian", type: "text", text: "• Idea: a clipboard ring you can rotate through" },
  { id: "9", app: "Terminal", type: "text", text: "cargo tauri build --release" },
  { id: "10", app: "Chrome", type: "text", text: "the cake is a lie (it was actually pretty good)" },
];

const TYPE_ICON: Record<Sample["type"], string> = {
  text: "📄",
  url: "🔗",
  image: "🖼",
};

export function LiveDemo() {
  const [q, setQ] = useState("");
  const [active, setActive] = useState(0);
  const [selected, setSelected] = useState<string[]>([]);
  const [output, setOutput] = useState<string | null>(null);

  const items = useMemo(() => {
    const needle = q.trim().toLowerCase();
    if (!needle) return SAMPLES;
    return SAMPLES.filter(
      (s) =>
        s.text.toLowerCase().includes(needle) || s.app.toLowerCase().includes(needle),
    );
  }, [q]);

  useEffect(() => {
    setActive((i) => Math.min(i, Math.max(0, items.length - 1)));
  }, [items.length]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Only handle if the demo search input is focused (or document has focus
      // and the user clearly intends the demo). For simplicity we bind globally
      // when the input is focused; ignore otherwise.
      const target = e.target as HTMLElement | null;
      if (target && target.id !== "demo-search") return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setActive((i) => Math.min(items.length - 1, i + 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setActive((i) => Math.max(0, i - 1));
      } else if (e.key === " " && items[active]) {
        e.preventDefault();
        const id = items[active].id;
        setSelected((prev) =>
          prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
        );
      } else if (e.key === "Enter" && items[active]) {
        e.preventDefault();
        runActivate(items[active]);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [items, active, selected]); // eslint-disable-line react-hooks/exhaustive-deps

  const runActivate = (clip: Sample) => {
    if (selected.length > 0 && selected.includes(clip.id)) {
      const merged = selected
        .map((id) => SAMPLES.find((s) => s.id === id)?.text ?? "")
        .filter(Boolean)
        .join(" ");
      setOutput(merged);
    } else {
      setOutput(clip.text);
      setSelected([]);
    }
  };

  const toggleSelect = (id: string) => {
    setSelected((prev) =>
      prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
    );
  };

  return (
    <section id="demo" className="relative py-20 sm:py-28">
      <div className="container-page">
        <div className="mx-auto max-w-2xl text-center">
          <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-accent">
            Try it
          </p>
          <h2 className="text-balance text-3xl font-bold tracking-tight sm:text-5xl">
            Click around. It's the real palette.
          </h2>
          <p className="mt-4 text-pretty text-base text-fg-muted sm:text-lg">
            Search a few letters, hit the arrows, tap <span className="kbd">␣</span> on
            a couple, then <span className="kbd">↵</span>. Watch the merge.
          </p>
        </div>

        <div className="mx-auto mt-12 grid max-w-5xl gap-6 lg:grid-cols-2 lg:gap-8">
          <div className="card p-0">
            <div className="flex items-center gap-2 border-b border-border px-4 py-3">
              <Search className="h-4 w-4 text-fg-muted" />
              <input
                id="demo-search"
                value={q}
                onChange={(e) => setQ(e.target.value)}
                placeholder="Search clips, URLs, code…"
                className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-subtle focus:outline-none"
              />
              {selected.length > 0 && (
                <span className="flex items-center gap-1 rounded-md bg-accent/20 px-2 py-0.5 text-[11px] font-medium text-accent">
                  <Combine className="h-3 w-3" />
                  {selected.length}
                </span>
              )}
              <span className="kbd">esc</span>
            </div>
            <ul className="max-h-[420px] overflow-y-auto p-1">
              {items.length === 0 ? (
                <li className="empty-state flex flex-col items-center gap-2 p-8 text-center text-sm text-fg-muted">
                  <Search className="h-7 w-7 opacity-30" />
                  <p>No matches</p>
                </li>
              ) : (
                items.map((c, i) => {
                  const isActive = i === active;
                  const isSel = selected.includes(c.id);
                  const order = selected.indexOf(c.id) + 1 || undefined;
                  return (
                    <li
                      key={c.id}
                      onMouseEnter={() => setActive(i)}
                      onClick={() => runActivate(c)}
                      data-active={isActive}
                      data-checked={isSel}
                      className={`group flex cursor-pointer items-center gap-3 rounded-md px-3 py-2.5 transition-colors ${
                        isActive ? "bg-accent/15 ring-1 ring-accent/40" : "hover:bg-bg-overlay"
                      } ${isSel ? "bg-accent/10" : ""}`}
                    >
                      <span
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleSelect(c.id);
                        }}
                        className="flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border border-border bg-bg-overlay text-[10px] text-accent"
                        title="Select for merge (or press Space)"
                      >
                        {isSel ? "✓" : ""}
                      </span>
                      <span className="grid h-7 w-7 shrink-0 place-items-center rounded-md bg-bg-overlay">
                        {TYPE_ICON[c.type]}
                      </span>
                      <div className="min-w-0 flex-1">
                        <div className="truncate font-mono text-sm text-fg">{c.text}</div>
                        <div className="text-[11px] text-fg-muted">{c.app}</div>
                      </div>
                      {order !== undefined && (
                        <span className="flex h-5 min-w-[20px] items-center justify-center rounded-full bg-accent px-1.5 text-[10px] font-bold text-white">
                          {order}
                        </span>
                      )}
                    </li>
                  );
                })
              )}
            </ul>
            <div className="flex items-center justify-between border-t border-border px-4 py-2 text-[11px] text-fg-muted">
              <div className="flex items-center gap-2">
                <span className="kbd">
                  <ArrowUp className="h-2.5 w-2.5" />
                </span>
                <span className="kbd">
                  <ArrowDown className="h-2.5 w-2.5" />
                </span>
                <span>navigate</span>
                <span className="kbd">␣</span>
                <span>select</span>
                <span className="kbd">
                  <CornerDownLeft className="h-2.5 w-2.5" />
                </span>
                <span>{selected.length > 0 ? "merge & paste" : "paste"}</span>
              </div>
              <span>
                {items.length} result{items.length === 1 ? "" : "s"}
              </span>
            </div>
          </div>

          <div className="card flex flex-col">
            <p className="text-xs font-semibold uppercase tracking-widest text-fg-subtle">
              Output preview
            </p>
            <h3 className="mt-1 text-lg font-semibold">What you'd paste</h3>
            <p className="mt-1 text-sm text-fg-muted">
              Click a row, or select a few and press Enter.
            </p>
            <div className="mt-4 flex-1 rounded-lg border border-border bg-bg-overlay/60 p-4 font-mono text-sm text-fg">
              {output ?? (
                <span className="text-fg-subtle">
                  …waiting for a clip. Try selecting two and hitting Enter to see the merge.
                </span>
              )}
            </div>
            <div className="mt-4 grid grid-cols-3 gap-2 text-[11px] text-fg-muted">
              <Stat label="clips" value={SAMPLES.length.toString()} />
              <Stat label="sources" value={new Set(SAMPLES.map((s) => s.app)).size.toString()} />
              <Stat label="search" value="fuzzy" />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-border bg-bg-overlay/40 px-3 py-2">
      <div className="text-[10px] uppercase tracking-widest text-fg-subtle">{label}</div>
      <div className="font-mono text-sm text-fg">{value}</div>
    </div>
  );
}
