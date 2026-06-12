import { useState } from "react";
import { useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Folder, Plus, Trash2, AlertCircle } from "lucide-react";
import { api } from "@/lib/tauri";
import { ClipRow } from "@/components/ClipRow";
import { collectionSchema } from "@/lib/schemas";

export function CollectionsPage() {
  const { id } = useParams<{ id?: string }>();
  const qc = useQueryClient();
  const [name, setName] = useState("");
  const [nameError, setNameError] = useState<string | null>(null);

  const collections = useQuery({
    queryKey: ["collections"],
    queryFn: api.listCollections,
  });

  const selected = id ?? null;

  const clips = useQuery({
    queryKey: ["clips", "collection", selected],
    queryFn: () => api.listClips({ collection_id: selected!, limit: 500 }),
    enabled: !!selected,
  });

  const create = useMutation({
    mutationFn: () => {
      const parsed = collectionSchema.safeParse({ name: name.trim(), icon: null });
      if (!parsed.success) {
        setNameError(parsed.error.issues[0]?.message ?? "Invalid name");
        throw new Error("invalid collection name");
      }
      setNameError(null);
      return api.createCollection(parsed.data.name, parsed.data.icon);
    },
    onSuccess: () => {
      setName("");
      setNameError(null);
      qc.invalidateQueries({ queryKey: ["collections"] });
    },
    onError: (err) => {
      // Re-thrown validation errors land here too; only log genuine server errors.
      if (err instanceof Error && err.message !== "invalid collection name") {
        console.error("Failed to create collection", err);
      }
    },
  });

  const remove = useMutation({
    mutationFn: (cid: string) => api.deleteCollection(cid),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["collections"] }),
  });

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border px-6 py-3">
        <div className="flex items-center gap-2">
          <Folder className="h-5 w-5" />
          <h2 className="text-lg font-semibold">Collections</h2>
        </div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            create.mutate();
          }}
          className="flex items-start gap-2"
        >
          <div className="flex flex-col">
            <input
              value={name}
              onChange={(e) => {
                setName(e.target.value);
                if (nameError) setNameError(null);
              }}
              placeholder="New collection..."
              className={`input max-w-[200px] ${nameError ? "border-danger" : ""}`}
              aria-invalid={!!nameError}
            />
            {nameError && (
              <span className="mt-1 inline-flex items-center gap-1 text-xs text-danger">
                <AlertCircle className="h-3 w-3" /> {nameError}
              </span>
            )}
          </div>
          <button className="btn-primary" type="submit" disabled={!name.trim()}>
            <Plus className="h-4 w-4" /> Create
          </button>
        </form>
      </header>
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {!selected ? (
          collections.data && collections.data.length === 0 ? (
            <div className="empty-state">
              <Folder className="h-8 w-8 opacity-30" />
              <p>No collections yet. Create one above.</p>
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
              {collections.data?.map((c) => (
                <div
                  key={c.id}
                  className="group relative rounded-md border border-border bg-bg-elevated p-4 transition-colors hover:border-accent/40"
                >
                  <a href={`#/collections/${c.id}`} className="block">
                    <Folder className="h-8 w-8 text-accent" />
                    <h3 className="mt-2 font-semibold">{c.name}</h3>
                    <p className="text-xs text-fg-muted">{c.clip_count} clips</p>
                  </a>
                  <button
                    className="absolute right-2 top-2 btn-ghost text-danger opacity-0 group-hover:opacity-100"
                    onClick={() => {
                      if (confirm(`Delete collection "${c.name}"?`)) remove.mutate(c.id);
                    }}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )
        ) : clips.data && clips.data.items.length === 0 ? (
          <div className="empty-state">No clips in this collection yet.</div>
        ) : (
          <div className="flex flex-col gap-1">
            {clips.data?.items.map((c) => (
              <ClipRow key={c.id} clip={c} showCollection />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
