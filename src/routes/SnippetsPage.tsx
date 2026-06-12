import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Code2, Plus, Trash2, Copy as CopyIcon, Star, Eye, Pencil, Search } from "lucide-react";
import { api } from "@/lib/tauri";
import { SUPPORTED_LANGUAGES } from "@/lib/languages";
import { CodePreview } from "@/components/CodePreview";

export function SnippetsPage() {
  const qc = useQueryClient();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [language, setLanguage] = useState("typescript");
  const [body, setBody] = useState("");
  const [isFavorite, setIsFavorite] = useState(false);
  const [mode, setMode] = useState<"edit" | "preview">("edit");
  const [searchInput, setSearchInput] = useState("");
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    const t = setTimeout(() => setSearchQuery(searchInput), 80);
    return () => clearTimeout(t);
  }, [searchInput]);

  const snippets = useQuery({
    queryKey: ["snippets", searchQuery],
    queryFn: () => (searchQuery ? api.searchSnippets(searchQuery, 200) : api.listSnippets()),
  });

  const editing = useMemo(
    () => snippets.data?.find((s) => s.id === editingId) ?? null,
    [snippets.data, editingId]
  );

  useEffect(() => {
    if (editing) {
      setTitle(editing.title);
      setLanguage(editing.language);
      setBody(editing.body);
      setIsFavorite(editing.is_favorite);
    } else {
      setTitle("");
      setLanguage("typescript");
      setBody("");
      setIsFavorite(false);
    }
    setMode("edit");
  }, [editing]);

  const save = useMutation({
    mutationFn: () =>
      api.upsertSnippet({
        id: editingId ?? undefined,
        title: title || "Untitled",
        language,
        body,
        is_favorite: isFavorite,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["snippets"] });
      setEditingId(null);
    },
    onError: (err) => {
      console.error("Failed to save snippet", err);
    },
  });

  const remove = useMutation({
    mutationFn: (id: string) => api.deleteSnippet(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["snippets"] }),
  });

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-2 border-b border-border px-6 py-3">
        <Code2 className="h-5 w-5" />
        <h2 className="text-lg font-semibold">Snippets</h2>
        <span className="text-sm text-fg-muted">{snippets.data?.length ?? 0} saved</span>
      </header>
      <div className="grid flex-1 grid-cols-[280px_1fr] overflow-hidden">
        <aside className="overflow-y-auto border-r border-border p-3">
          <button
            className="btn-primary mb-3 w-full"
            onClick={() => {
              setEditingId(null);
              setTitle("");
              setLanguage("typescript");
              setBody("");
              setIsFavorite(false);
              setMode("edit");
            }}
          >
            <Plus className="h-4 w-4" /> New snippet
          </button>
          <div className="relative mb-3">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-fg-muted" />
            <input
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search snippets..."
              className="input w-full pl-8"
            />
          </div>
          <div className="flex flex-col gap-0.5">
            {snippets.data && snippets.data.length === 0 && (
              <p className="px-2 py-4 text-center text-xs text-fg-muted">
                {searchQuery ? "No matches" : "No snippets yet. Create one with the + button."}
              </p>
            )}
            {snippets.data?.map((s) => (
              <div
                key={s.id}
                data-active={s.id === editingId}
                className="sidebar-item"
                onClick={() => setEditingId(s.id)}
              >
                <span className="truncate">{s.title}</span>
                <span className="text-[10px] text-fg-subtle">{s.language}</span>
              </div>
            ))}
          </div>
        </aside>
        <div className="flex flex-col overflow-hidden">
          <div className="flex items-center gap-2 border-b border-border p-3">
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Title"
              className="input flex-1"
            />
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              className="input max-w-[160px]"
            >
              {SUPPORTED_LANGUAGES.map((l) => (
                <option key={l} value={l}>
                  {l}
                </option>
              ))}
            </select>
            <button
              className="btn-ghost"
              onClick={() => setIsFavorite((v) => !v)}
              title="Favorite"
            >
              <Star
                className={`h-4 w-4 ${isFavorite ? "fill-warning text-warning" : ""}`}
              />
            </button>
            {editing && (
              <button
                className="btn-ghost text-danger"
                onClick={() => editing && remove.mutate(editing.id)}
                title="Delete"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            )}
            <button
              className="btn-ghost"
              onClick={() => editing && api.copySnippetToClipboard(editing.id)}
              title="Copy to clipboard"
              disabled={!editing || !body.trim()}
            >
              <CopyIcon className="h-4 w-4" />
            </button>
            <div className="ml-1 flex overflow-hidden rounded-md border border-border">
              <button
                className={`px-2 py-1 text-xs ${mode === "edit" ? "bg-accent text-accent-fg" : "text-fg-muted"}`}
                onClick={() => setMode("edit")}
                title="Edit"
              >
                <Pencil className="h-3 w-3" />
              </button>
              <button
                className={`px-2 py-1 text-xs ${mode === "preview" ? "bg-accent text-accent-fg" : "text-fg-muted"}`}
                onClick={() => setMode("preview")}
                title="Preview"
              >
                <Eye className="h-3 w-3" />
              </button>
            </div>
            <button
              className="btn-primary"
              onClick={() => save.mutate()}
              disabled={!body.trim() || save.isPending}
            >
              {save.isPending ? "Saving…" : "Save"}
            </button>
          </div>
          {mode === "edit" ? (
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              className="flex-1 resize-none bg-bg p-4 font-mono text-sm text-fg focus:outline-none"
              placeholder="// your snippet here"
            />
          ) : (
            <div className="flex-1 overflow-auto bg-bg p-4">
              <CodePreview code={body} language={language} maxHeight={undefined} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
