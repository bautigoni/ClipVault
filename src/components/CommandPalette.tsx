import { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { Search, FileText, Image as ImageIcon, Files, Link2, Star, Pin, Combine, X, Wand2, ZoomIn } from "lucide-react";
import { api, type TextTransformKind, type TransformResult } from "@/lib/tauri";
import { debounce, relativeDateGroup } from "@/lib/utils";
import type { Clip } from "@/types";

const TYPE_ICONS = {
  text: FileText,
  image: ImageIcon,
  files: Files,
  url: Link2,
} as const;

export function CommandPalette() {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  // Multi-select: an ordered array of clip ids. Order is the order the user
  // toggled them with Space, which is also the order the backend will
  // concatenate them in. We keep a Set alongside for O(1) membership checks
  // in the row renderer.
  const [selected, setSelected] = useState<string[]>([]);
  const selectedSet = useMemo(() => new Set(selected), [selected]);
  const inputRef = useRef<HTMLInputElement>(null);
  // Latest values for the global keydown handler (which is registered once
  // and must not capture stale state via stale closure).
  const stateRef = useRef({ items: [] as Clip[], active: 0, selected: [] as string[], query: "" });
  const qc = useQueryClient();

  const debouncedSetQuery = useRef(debounce(setQuery, 30)).current;

  const results = useQuery({
    queryKey: ["palette", query],
    queryFn: () =>
      api.searchClips({
        query: query || undefined,
        limit: 50,
      }),
  });

  // Pull the user's settings so the keyboard navigation can honor their
  // configured Ctrl+↑/↓ jump size. We don't strictly need the rest of the
  // settings here, but loading the whole object is cheap and keeps the
  // surface narrow.
  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => api.getSettings(),
  });
  const jumpSize = settingsQuery.data?.palette_jump_size ?? 0;

  const items = results.data?.items ?? [];

  // Keep the latest snapshot in a ref so the global keydown handler (registered
  // exactly once below) can read the freshest values without re-binding.
  useEffect(() => {
    stateRef.current = { items, active, selected, query };
  }, [items, active, selected, query]);

  // When the active row changes (↑/↓/Ctrl+↑↓/mouse hover), bring it into
  // view if it falls outside the scroll container. `block: "nearest"` is
  // the gentlest option — it only scrolls when the row would otherwise be
  // clipped, so the list doesn't jump on every keystroke.
  useEffect(() => {
    if (items.length === 0) return;
    const el = document.querySelector(`[data-palette-row="${active}"]`);
    if (el && "scrollIntoView" in el) {
      (el as HTMLElement).scrollIntoView({ block: "nearest", behavior: "auto" });
    }
  }, [active, items.length]);

  const toggleSelect = useCallback((id: string) => {
    setSelected((prev) => {
      const idx = prev.indexOf(id);
      if (idx >= 0) {
        // Deselect: splice it out, preserving the order of the rest.
        const next = prev.slice();
        next.splice(idx, 1);
        return next;
      }
      // Select: append to the end so the merge order matches the order the
      // user toggled (first selected → first pasted, etc).
      return [...prev, id];
    });
  }, []);

  const clearSelection = useCallback(() => setSelected([]), []);

  // The Enter / click handler dispatches to either the merge command (when
  // there's a multi-selection) or the single-clip copy command. The backend
  // honours the `auto_paste` setting for both, so a single Enter key is
  // enough to copy-and-paste in 99% of cases.
  const activate = useCallback(
    async (clip: Clip) => {
      const { selected: sel } = stateRef.current;
      try {
        if (sel.length >= 1 && sel.includes(clip.id)) {
          // Pass the selection straight through — the array is already in
          // the order the user picked the clips with Space, which is the
          // order the backend concatenates in.
          await api.mergeAndPasteClips(sel);
        } else {
          await api.copyClipToClipboard(clip.id);
        }
      } catch (err) {
        console.error("activate failed", err);
      } finally {
        await api.hidePalette();
      }
    },
    [],
  );

  // Global keydown handler. Listening on `window` (not the input) means the
  // palette still responds to ↑/↓/Enter/Space/Esc even when the search box
  // loses focus (which can happen when a row is hovered/clicked or when the
  // polling refresh re-renders the list). We keep the input focused for
  // typing, but treat the whole window as the keyboard capture surface.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const { items: list, active: a, selected: sel } = stateRef.current;
      if (e.key === "Escape") {
        e.preventDefault();
        if (sel.length > 0) {
          clearSelection();
        } else {
          api.hidePalette();
        }
        return;
      }
      if (e.key === "ArrowDown") {
        e.preventDefault();
        // Ctrl+↓ jumps: 0 = bottom of the list, otherwise jump by N rows.
        if (e.ctrlKey) {
          const target = jumpSize === 0 ? list.length - 1 : Math.min(list.length - 1, a + jumpSize);
          setActive(target);
        } else {
          setActive((i) => Math.min(list.length - 1, i + 1));
        }
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        // Ctrl+↑ jumps: 0 = top of the list, otherwise jump by N rows.
        if (e.ctrlKey) {
          const target = jumpSize === 0 ? 0 : Math.max(0, a - jumpSize);
          setActive(target);
        } else {
          setActive((i) => Math.max(0, i - 1));
        }
        return;
      }
      if (e.key === " " && list[a]) {
        // Space toggles selection of the current row. preventDefault keeps
        // the space from ending up in the search box.
        const target = e.target as HTMLElement | null;
        if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA")) {
          e.preventDefault();
        }
        toggleSelect(list[a].id);
        return;
      }
      if (e.key === "Enter" && list[a]) {
        e.preventDefault();
        activate(list[a]);
        return;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activate, clearSelection, toggleSelect, jumpSize]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Auto-refresh: invalidate the palette query whenever the Rust backend
  // reports a new or updated clip. We also force a refetch on mount (so the
  // list is current every time the palette is shown) and run a small
  // polling loop as a safety net — clipboard events from a different
  // webview can be missed by a `listen()` that wasn't registered in time.
  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];
    const refresh = () => {
      if (!cancelled) qc.invalidateQueries({ queryKey: ["palette"] });
    };

    // 1) Immediate refetch on mount/visibility. This guarantees a fresh
    //    result set every time the palette is shown.
    refresh();

    // 2) Listen for the Rust-emitted events as the primary path.
    listen("clip://created", refresh)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://created listen failed", err));
    listen("clip://updated", refresh)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://updated listen failed", err));

    // 3) Polling fallback: 2s is short enough to feel instant when the
    //    user copies something and walks to the palette, and cheap enough
    //    that it doesn't matter while the palette is open.
    const interval = window.setInterval(() => {
      if (!cancelled) refresh();
    }, 2000);

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
      window.clearInterval(interval);
    };
  }, [qc]);

  // Reset cursor + selection whenever the *search input* changes. We
  // intentionally don't reset on every refetch — the polling loop can swap
  // the data in the background, and resetting the cursor then makes arrow
  // navigation feel jumpy.
  useEffect(() => {
    setActive(0);
    setSelected([]);
  }, [query]);

  // Order index of each selected clip id (1-based) so the row can show a
  // "1, 2, 3" badge that matches the order the user toggled with Space. This
  // is also the order the backend concatenates in.
  const orderIndex = useMemo(() => {
    const m = new Map<string, number>();
    selected.forEach((id, i) => m.set(id, i + 1));
    return m;
  }, [selected]);

  return (
    <div className="flex h-full w-full flex-col rounded-xl border border-border bg-bg-elevated/95 shadow-2xl backdrop-blur">
      <div className="flex items-center gap-2 border-b border-border px-4 py-3">
        <Search className="h-4 w-4 text-fg-muted" />
        <input
          ref={inputRef}
          type="text"
          className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-subtle focus:outline-none"
          placeholder="Search clips, URLs, code..."
          onChange={(e) => debouncedSetQuery(e.target.value)}
          onFocus={() => {
            /* keep the global keydown listener in charge of navigation */
          }}
        />
        {selected.length > 0 && (
          <span
            data-testid="selection-badge"
            className="flex items-center gap-1 rounded-md bg-accent/20 px-2 py-0.5 text-[11px] font-medium text-accent"
          >
            <Combine className="h-3 w-3" />
            {selected.length} selected
            <button
              type="button"
              aria-label="Clear selection"
              onClick={clearSelection}
              className="ml-1 rounded p-0.5 hover:bg-accent/30"
            >
              <X className="h-3 w-3" />
            </button>
          </span>
        )}
        <span className="kbd">esc</span>
      </div>
      <div
        className="flex-1 overflow-y-auto p-1"
        // Prevent the list from stealing keyboard focus on click, so the
        // global keydown handler (and the search input) keep working.
        onMouseDown={(e) => e.preventDefault()}
      >
        {items.length === 0 ? (
          <div className="empty-state">
            <Search className="h-8 w-8 opacity-30" />
            <p className="text-sm">{query ? "No matches" : "Start typing to search..."}</p>
          </div>
        ) : (
          items.map((clip, i) => (
            <PaletteRow
              key={clip.id}
              clip={clip}
              index={i}
              active={i === active}
              checked={selectedSet.has(clip.id)}
              order={orderIndex.get(clip.id)}
              onHover={() => setActive(i)}
              onToggleCheck={() => toggleSelect(clip.id)}
              onClick={() => activate(clip)}
            />
          ))
        )}
      </div>
      <div className="flex items-center justify-between border-t border-border px-4 py-2 text-[11px] text-fg-muted">
        <div className="flex items-center gap-2">
          <span className="kbd">↑↓</span>
          <span>navigate</span>
          <span className="kbd">ctrl+↑↓</span>
          <span>{jumpSize === 0 ? "top/bottom" : `jump ${jumpSize}`}</span>
          <span className="kbd">␣</span>
          <span>select</span>
          <span className="kbd">↵</span>
          <span>{selected.length > 0 ? "merge & paste" : "paste"}</span>
          <span className="kbd">esc</span>
          <span>close</span>
        </div>
        <span>{results.data?.total ?? 0} results · {results.data?.took_ms ?? 0}ms</span>
      </div>
    </div>
  );
}

function PaletteRow({
  clip,
  index,
  active,
  checked,
  order,
  onHover,
  onToggleCheck,
  onClick,
}: {
  clip: Clip;
  index: number;
  active: boolean;
  checked: boolean;
  /** 1-based position in the merge order, or undefined if not selected. */
  order: number | undefined;
  onHover: () => void;
  onToggleCheck: () => void;
  onClick: () => void;
}) {
  const Icon = TYPE_ICONS[clip.type];
  const group = relativeDateGroup(clip.created_at);
  return (
    <div
      data-palette-row={index}
      data-active={active}
      data-checked={checked}
      onMouseEnter={onHover}
      onMouseDown={(e) => {
        e.preventDefault();
        // If the user shift-clicks, treat it as a multi-select toggle
        // instead of a primary action — matches the keyboard space behavior.
        if (e.shiftKey) {
          onToggleCheck();
        } else {
          onClick();
        }
      }}
      className="clip-row cursor-pointer"
    >
      <div className="flex h-5 w-5 shrink-0 items-center justify-center">
        <input
          type="checkbox"
          checked={checked}
          onChange={onToggleCheck}
          onClick={(e) => e.stopPropagation()}
          aria-label={`Select clip ${clip.text_preview?.slice(0, 32) ?? clip.id}`}
          className="h-3.5 w-3.5 cursor-pointer accent-accent"
        />
      </div>
      {order !== undefined && (
        <span
          aria-label={`Merge order ${order}`}
          className="flex h-5 min-w-[20px] shrink-0 items-center justify-center rounded-full bg-accent px-1.5 text-[10px] font-bold text-accent-fg"
        >
          {order}
        </span>
      )}
      <div className="grid h-8 w-8 shrink-0 place-items-center overflow-hidden rounded-md bg-bg-overlay text-fg-muted">
        {clip.type === "image" && clip.image ? (
          <ImageThumb clip={clip} />
        ) : (
          <Icon className="h-4 w-4" />
        )}
      </div>
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="truncate text-sm text-fg">
          {clip.type === "image"
            ? `Image · ${clip.image?.width ?? "?"}×${clip.image?.height ?? "?"}`
            : clip.text_preview || `[${clip.type}]`}
        </div>
        <div className="flex items-center gap-2 text-[11px] text-fg-muted">
          {clip.source_app && <span>{clip.source_app}</span>}
          <span>·</span>
          <span>{group}</span>
        </div>
      </div>
      <div className="flex items-center gap-1 text-fg-muted">
        {clip.is_pinned && <Pin className="h-3.5 w-3.5" />}
        {clip.is_favorite && <Star className="h-3.5 w-3.5 fill-warning text-warning" />}
        {(clip.type === "text" || clip.type === "url") && (
          <TransformMenu clipId={clip.id} preview={clip.text_preview ?? ""} />
        )}
      </div>
      {clip.type === "image" && clip.image && (
        <ImagePreviewPopover clip={clip} />
      )}
    </div>
  );
}

/// Hover-preview popover for image clips. Shows the FULL image (not the
/// thumbnail) on hover, so the user can actually see the screenshot they
/// copied without having to paste it somewhere first. The full image bytes
/// are loaded on demand and revoked on unmount to avoid leaking memory.
function ImagePreviewPopover({ clip }: { clip: Clip }) {
  const [url, setUrl] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Lazy-load the full image the first time the user actually hovers the
  // popover trigger. This keeps the palette fast when scrolling: only the
  // visible image rows ever pay the cost of fetching the full image.
  useEffect(() => {
    if (!open || url || loading || !clip.image) return;
    let active = true;
    setLoading(true);
    setError(null);
    (async () => {
      try {
        // Tauri serializes `Vec<u8>` as `number[]` on the JS side. Build
        // a Uint8Array from it explicitly so we get a real byte buffer
        // we can wrap in a Blob, regardless of what Tauri hands us.
        const raw = await api.readImageFull(clip.image!.path);
        if (!active) return;
        if (!raw) {
          throw new Error("readImageFull returned empty payload");
        }
        const arr = raw instanceof ArrayBuffer
          ? new Uint8Array(raw)
          : Array.isArray(raw)
            ? Uint8Array.from(raw as number[])
            : new Uint8Array(raw as ArrayLike<number>);
        if (arr.byteLength === 0) {
          throw new Error("readImageFull returned 0 bytes");
        }
        const mime = clip.image?.mime === "image/png" ? "image/png" : "image/jpeg";
        const blob = new Blob([arr], { type: mime });
        const objectUrl = URL.createObjectURL(blob);
        if (active) setUrl(objectUrl);
      } catch (e: any) {
        // eslint-disable-next-line no-console
        console.warn("[ImagePreviewPopover] failed to load full image", {
          path: clip.image?.path,
          error: e,
        });
        if (active) setError(String(e?.message ?? e));
      } finally {
        if (active) setLoading(false);
      }
    })();
    return () => {
      active = false;
    };
  }, [open, url, loading, clip.image]);

  // Revoke on full unmount only.
  useEffect(() => {
    return () => {
      if (url) URL.revokeObjectURL(url);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!clip.image) return null;
  return (
    <div
      ref={containerRef}
      className="relative ml-2"
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      <button
        type="button"
        title="Preview image"
        onClick={(e) => {
          e.stopPropagation();
          setOpen((v) => !v);
        }}
        onMouseDown={(e) => e.stopPropagation()}
        className="rounded p-1 text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
      >
        <ZoomIn className="h-3.5 w-3.5" />
      </button>
      {open && (
        <div
          className="absolute right-0 top-full z-50 mt-1 max-h-[70vh] max-w-[min(640px,80vw)] overflow-auto rounded-md border border-border bg-bg-elevated p-2 shadow-2xl"
          onMouseDown={(e) => e.stopPropagation()}
        >
          {loading && !url && (
            <div className="flex h-40 w-72 items-center justify-center text-xs text-fg-muted">
              Loading full image…
            </div>
          )}
          {url ? (
            <img
              src={url}
              alt={clip.text_preview ?? "clip image"}
              className="block max-w-full rounded"
              draggable={false}
            />
          ) : !loading ? (
            <div className="flex h-40 w-72 flex-col items-center justify-center gap-1 px-3 text-center text-xs text-fg-muted">
              <span>Couldn't load full image</span>
              {error && (
                <span className="max-w-full truncate text-[10px] opacity-70" title={error}>
                  {error}
                </span>
              )}
            </div>
          ) : null}
          {clip.image && (
            <div className="mt-2 flex items-center justify-between text-[10px] text-fg-muted">
              <span>
                {clip.image.width}×{clip.image.height}
              </span>
              <span className="truncate">{clip.source_app ?? "unknown source"}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/// Tiny "transform" menu that runs a clip's text through one of the cheap
/// text transformations exposed by the backend. We keep the UI minimal
/// (no nested submenus) — the most-used ones live at the top in the order
/// users asked for them in the user research.
function TransformMenu({ clipId, preview }: { clipId: string; preview: string }) {
  const [open, setOpen] = useState(false);
  const [result, setResult] = useState<TransformResult | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  const run = async (kind: TextTransformKind) => {
    setOpen(false);
    try {
      // We re-fetch the full text because `preview` is truncated to 200
      // chars in the list. The DB is the source of truth.
      const full = await api.getClip(clipId);
      const text = full?.text_preview ?? preview;
      const r = await api.transformClip(text, kind);
      setResult(r);
    } catch (e) {
      console.error("transform failed", e);
    }
  };

  const copy = async () => {
    if (!result) return;
    // Push the result to the system clipboard so the user can paste it
    // anywhere. We deliberately don't auto-paste — the user is in control.
    await navigator.clipboard.writeText(result.text);
    setResult(null);
  };

  const insertAsNew = async () => {
    if (!result) return;
    // Insert the result back into the DB as a new text clip. We use the
    // existing copyClipToClipboard flow indirectly: the watcher will pick
    // up the clipboard change as a brand-new text clip on its next poll,
    // preserving the dedupe + source-app + OCR pipeline.
    await navigator.clipboard.writeText(result.text);
    setResult(null);
  };

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        title="Transform text"
        onClick={(e) => {
          e.stopPropagation();
          setOpen((v) => !v);
        }}
        onMouseDown={(e) => e.stopPropagation()}
        className="rounded p-1 text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
      >
        <Wand2 className="h-3.5 w-3.5" />
      </button>
      {open && (
        <div
          className="absolute right-0 top-full z-50 mt-1 w-48 overflow-hidden rounded-md border border-border bg-bg-elevated shadow-lg"
          onMouseDown={(e) => e.stopPropagation()}
        >
          {(
            [
              ["uppercase", "UPPERCASE"],
              ["lowercase", "lowercase"],
              ["title_case", "Title Case"],
              ["sentence_case", "Sentence case"],
              ["trim", "Trim"],
              ["collapse_whitespace", "Collapse whitespace"],
              ["dedup_lines", "Dedupe lines"],
              ["unique_lines", "Unique lines"],
              ["sort_lines_asc", "Sort A→Z"],
              ["sort_lines_desc", "Sort Z→A"],
              ["strip_empty_lines", "Strip empty lines"],
              ["to_single_line", "To single line"],
              ["reverse", "Reverse"],
              ["url_encode", "URL encode"],
              ["url_decode", "URL decode"],
              ["base64_encode", "Base64 encode"],
              ["base64_decode", "Base64 decode"],
              ["count", "Count…"],
            ] as Array<[TextTransformKind, string]>
          ).map(([kind, label]) => (
            <button
              key={kind}
              onClick={() => run(kind)}
              className="block w-full px-3 py-1.5 text-left text-xs text-fg hover:bg-bg-overlay"
            >
              {label}
            </button>
          ))}
        </div>
      )}
      {result && (
        <TransformResultPopover
          result={result}
          onCopy={copy}
          onInsertAsNew={insertAsNew}
          onClose={() => setResult(null)}
        />
      )}
    </div>
  );
}

function TransformResultPopover({
  result,
  onCopy,
  onInsertAsNew,
  onClose,
}: {
  result: TransformResult;
  onCopy: () => void;
  onInsertAsNew: () => void;
  onClose: () => void;
}) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl overflow-hidden rounded-lg border border-border bg-bg-elevated shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-border bg-bg-overlay/50 px-4 py-2">
          <div className="text-sm font-semibold text-fg">
            {result.label}{" "}
            <span className="font-normal text-fg-muted">
              · {result.in_len} → {result.out_len}
            </span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-fg-muted hover:bg-bg-overlay hover:text-fg"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        <div className="max-h-96 overflow-auto p-4">
          {result.counts ? (
            <dl className="grid grid-cols-2 gap-3 text-sm text-fg">
              <div>
                <dt className="text-xs uppercase text-fg-muted">Characters</dt>
                <dd className="font-mono text-lg">{result.counts.chars}</dd>
              </div>
              <div>
                <dt className="text-xs uppercase text-fg-muted">Words</dt>
                <dd className="font-mono text-lg">{result.counts.words}</dd>
              </div>
              <div>
                <dt className="text-xs uppercase text-fg-muted">Lines</dt>
                <dd className="font-mono text-lg">{result.counts.lines}</dd>
              </div>
              <div>
                <dt className="text-xs uppercase text-fg-muted">Bytes</dt>
                <dd className="font-mono text-lg">{result.counts.bytes}</dd>
              </div>
            </dl>
          ) : (
            <pre className="whitespace-pre-wrap break-words font-mono text-xs text-fg">
              {result.text}
            </pre>
          )}
        </div>
        <div className="flex items-center justify-end gap-2 border-t border-border bg-bg-overlay/50 px-4 py-2">
          <button
            type="button"
            onClick={onInsertAsNew}
            className="rounded-md border border-border bg-transparent px-3 py-1.5 text-xs text-fg hover:bg-bg-overlay"
          >
            Save as new clip
          </button>
          <button
            type="button"
            onClick={onCopy}
            className="rounded-md bg-accent px-3 py-1.5 text-xs font-semibold text-accent-fg hover:bg-accent/90"
          >
            Copy to clipboard
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Lazily load the clip's thumbnail as a blob URL. The bytes come from a Tauri
 * command (`read_image_thumb`) which returns a `number[]` (u8 array) over the
 * IPC bridge. We wrap them in a Blob and turn that into an object URL the
 * browser can <img src=...> directly. The URL is revoked on unmount to
 * avoid leaking memory in a long-lived palette session.
 */
function ImageThumb({ clip }: { clip: Clip }) {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    let createdUrl: string | null = null;
    if (!clip.image) return;
    api
      .readImageThumb(clip.image.thumb_path)
      .then((bytes) => {
        if (!active) return;
        const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : new Uint8Array(bytes);
        const mime = clip.image?.mime === "image/png" ? "image/png" : "image/jpeg";
        const blob = new Blob([arr], { type: mime });
        createdUrl = URL.createObjectURL(blob);
        setUrl(createdUrl);
      })
      .catch(() => {
        /* ignore — the row falls back to the icon */
      });
    return () => {
      active = false;
      if (createdUrl) URL.revokeObjectURL(createdUrl);
    };
  }, [clip.id, clip.image?.thumb_path, clip.image?.mime]);
  if (!url) {
    return <ImageIcon className="h-4 w-4" />;
  }
  return <img src={url} alt="" className="h-full w-full object-cover" draggable={false} />;
}
