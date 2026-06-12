import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";

/**
 * Subscribes to `clip://created` and `clip://updated` events from the Rust backend and
 * invalidates the relevant queries so the UI stays in sync without polling.
 */
export function useClipsInvalidation() {
  const qc = useQueryClient();
  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    const onCreated = () => {
      qc.invalidateQueries({ queryKey: ["clips"] });
      qc.invalidateQueries({ queryKey: ["collections"] });
    };
    const onUpdated = () => {
      qc.invalidateQueries({ queryKey: ["clips"] });
    };

    listen("clip://created", onCreated)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://created listen failed", err));

    listen("clip://updated", onUpdated)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://updated listen failed", err));

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, [qc]);
}
