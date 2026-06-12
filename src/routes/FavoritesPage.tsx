import { useQuery } from "@tanstack/react-query";
import { Star } from "lucide-react";
import { api } from "@/lib/tauri";
import { ClipRow } from "@/components/ClipRow";

export function FavoritesPage() {
  const clips = useQuery({
    queryKey: ["clips", "favorites"],
    queryFn: () => api.listClips({ favorites_only: true, limit: 500 }),
  });

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-2 border-b border-border px-6 py-3">
        <Star className="h-5 w-5 fill-warning text-warning" />
        <h2 className="text-lg font-semibold">Favorites</h2>
        <span className="text-sm text-fg-muted">{clips.data?.total ?? 0} clips</span>
      </header>
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {clips.data && clips.data.items.length === 0 ? (
          <div className="empty-state">
            <Star className="h-8 w-8 opacity-30" />
            <p>No favorites yet. Star a clip to pin it forever.</p>
          </div>
        ) : (
          <div className="flex flex-col gap-1">
            {clips.data?.items.map((c) => (
              <ClipRow key={c.id} clip={c} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
