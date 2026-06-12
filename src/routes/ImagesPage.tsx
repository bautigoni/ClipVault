import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Image as ImageIcon } from "lucide-react";
import { api } from "@/lib/tauri";
import type { Clip } from "@/types";

export function ImagesPage() {
  const clips = useQuery({
    queryKey: ["clips", "images"],
    queryFn: () => api.listClips({ kind: "image", limit: 500 }),
  });
  const [selected, setSelected] = useState<Clip | null>(null);
  const [fullSrc, setFullSrc] = useState<string | null>(null);

  useEffect(() => {
    if (!selected || !selected.image) {
      setFullSrc(null);
      return;
    }
    let active = true;
    let urlToRevoke: string | null = null;
    const path = selected.image.path;
    api
      .readImageFull(path)
      .then((bytes) => {
        if (!active) return;
        const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : new Uint8Array(bytes);
        const blob = new Blob([arr], { type: "image/png" });
        urlToRevoke = URL.createObjectURL(blob);
        setFullSrc(urlToRevoke);
      })
      .catch(() => {
        if (active) setFullSrc(null);
      });
    return () => {
      active = false;
      if (urlToRevoke) URL.revokeObjectURL(urlToRevoke);
    };
  }, [selected]);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-2 border-b border-border px-6 py-3">
        <ImageIcon className="h-5 w-5" />
        <h2 className="text-lg font-semibold">Images</h2>
        <span className="text-sm text-fg-muted">{clips.data?.total ?? 0} saved</span>
      </header>
      <div className="flex-1 overflow-y-auto p-6">
        {clips.data && clips.data.items.length === 0 ? (
          <div className="empty-state">
            <ImageIcon className="h-8 w-8 opacity-30" />
            <p>No images yet. Copy an image (e.g. Print Screen) to save it here.</p>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
            {clips.data?.items.map((c) => (
              <ImageTile key={c.id} clip={c} onClick={() => setSelected(c)} />
            ))}
          </div>
        )}
      </div>
      {selected && (
        <div
          className="fixed inset-0 z-50 grid place-items-center bg-black/70 backdrop-blur"
          onClick={() => setSelected(null)}
        >
          <div className="max-w-[90vw] max-h-[90vh]">
            {fullSrc ? (
              <img src={fullSrc} alt="" className="max-h-[90vh] rounded-lg" />
            ) : (
              <div className="grid h-64 w-96 place-items-center text-fg-muted">Loading…</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function ImageTile({ clip, onClick }: { clip: Clip; onClick: () => void }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    if (!clip.image) return;
    let active = true;
    let urlToRevoke: string | null = null;
    const path = clip.image.thumb_path;
    api
      .readImageThumb(path)
      .then((bytes) => {
        if (!active) return;
        const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : new Uint8Array(bytes);
        const blob = new Blob([arr], { type: "image/jpeg" });
        urlToRevoke = URL.createObjectURL(blob);
        setSrc(urlToRevoke);
      })
      .catch(() => {});
    return () => {
      active = false;
      if (urlToRevoke) URL.revokeObjectURL(urlToRevoke);
    };
  }, [clip.image?.thumb_path]);
  return (
    <button
      onClick={onClick}
      className="group relative aspect-square overflow-hidden rounded-md border border-border bg-bg-overlay transition-transform hover:scale-[1.02]"
    >
      {src ? (
        <img src={src} alt="" className="h-full w-full object-cover" />
      ) : (
        <div className="grid h-full w-full place-items-center text-fg-muted">…</div>
      )}
    </button>
  );
}
