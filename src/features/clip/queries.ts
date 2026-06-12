import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type ClipFilter } from "@/lib/tauri";

/**
 * Feature-sliced hook for clip queries. Wraps the Tauri command surface with
 * TanStack Query so every page that needs clips shares the same cache key and
 * invalidation behavior.
 */

export const clipKeys = {
  all: ["clips"] as const,
  list: (filter: ClipFilter) => ["clips", "list", filter] as const,
  detail: (id: string) => ["clips", "detail", id] as const,
};

export function useClips(filter: ClipFilter) {
  return useQuery({
    queryKey: clipKeys.list(filter),
    // Route to searchClips when a query is present so the FTS5 pipeline runs;
    // otherwise listClips (which doesn't accept `query` in Rust).
    queryFn: () =>
      filter.query
        ? api.searchClips(filter)
        : api.listClips(filter),
  });
}

export function useClip(id: string | null) {
  return useQuery({
    queryKey: id ? clipKeys.detail(id) : ["clips", "detail", "none"],
    queryFn: () => (id ? api.getClip(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useToggleFavorite() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, value }: { id: string; value: boolean }) =>
      api.toggleFavorite(id, value),
    onSuccess: () => qc.invalidateQueries({ queryKey: clipKeys.all }),
  });
}

export function useDeleteClip() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteClip(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: clipKeys.all }),
  });
}

export function useAssignToCollection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ clipId, collectionId }: { clipId: string; collectionId: string | null }) =>
      api.assignToCollection(clipId, collectionId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: clipKeys.all });
      qc.invalidateQueries({ queryKey: ["collections"] });
    },
  });
}
