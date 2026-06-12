import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, type RingSlotView } from "@/lib/tauri";

export interface RingRotatedEvent {
  id: string;
  index: number;
  total: number;
  no_op?: boolean;
}

/**
 * Subscribes to `clip://ring-rotated`, `clip://ring-empty`,
 * `clip://ring-preview`, and `clip://ring-dismissed` events from the Rust
 * backend. Exposes the latest preview / status so the palette and overlay
 * components can render it.
 */
export function useRing() {
  const [active, setActive] = useState<RingRotatedEvent | null>(null);
  const [preview, setPreview] = useState<RingSlotView[]>([]);
  const [dismissed, setDismissed] = useState(false);
  const [empty, setEmpty] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    const onRotated = (e: { payload: RingRotatedEvent }) => {
      if (cancelled) return;
      if (e.payload.no_op) {
        // No-op rotation (clipboard already matches the slot) — keep the
        // previous active state rather than overriding with a no-op.
        return;
      }
      setActive(e.payload);
      setEmpty(false);
      setDismissed(false);
    };
    const onEmpty = () => {
      if (cancelled) return;
      setEmpty(true);
      setActive(null);
    };
    const onPreview = (e: { payload: RingSlotView[] }) => {
      if (cancelled) return;
      setPreview(e.payload);
    };
    const onDismissed = () => {
      if (cancelled) return;
      setDismissed(true);
      setActive(null);
    };

    listen("clip://ring-rotated", onRotated)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://ring-rotated listen failed", err));

    listen("clip://ring-empty", onEmpty)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://ring-empty listen failed", err));

    listen("clip://ring-preview", onPreview)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://ring-preview listen failed", err));

    listen("clip://ring-dismissed", onDismissed)
      .then((u) => {
        if (cancelled) u();
        else unlisteners.push(u);
      })
      .catch((err) => console.error("clip://ring-dismissed listen failed", err));

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  return { active, preview, dismissed, empty };
}

/**
 * Convenience wrapper that returns a function to manually refresh the preview
 * by calling the backend's `ring_preview` command.
 */
export function useRingPreview() {
  const [preview, setPreview] = useState<RingSlotView[]>([]);
  const refresh = async (n = 5) => {
    try {
      const slots = await api.ringPreview(n);
      setPreview(slots);
    } catch (e) {
      console.error("ringPreview failed", e);
    }
  };
  return { preview, refresh };
}
