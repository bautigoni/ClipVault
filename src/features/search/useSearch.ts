import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";

/**
 * Debounced search hook. Returns the latest result for the latest query.
 */
export function useSearch(initialQuery = "") {
  const [query, setQuery] = useState(initialQuery);
  const [debounced, setDebounced] = useState(initialQuery);

  useEffect(() => {
    const t = setTimeout(() => setDebounced(query), 30);
    return () => clearTimeout(t);
  }, [query]);

  const result = useQuery({
    queryKey: ["search", debounced],
    queryFn: () => api.searchClips({ query: debounced || undefined, limit: 50 }),
  });

  return {
    query,
    setQuery,
    debounced,
    items: result.data?.items ?? [],
    total: result.data?.total ?? 0,
    tookMs: result.data?.took_ms ?? 0,
    isLoading: result.isLoading,
  };
}
