import { useEffect, useState } from "react";
import { useRing } from "@/hooks/useRing";
import { api } from "@/lib/tauri";
import { ChevronLeft, ChevronRight, X, CircleAlert } from "lucide-react";

/**
 * Top-of-screen ring overlay. Renders a slim bar with the next ~5 ring slots,
 * highlighting the currently active one. Dismissable via Esc or the X button.
 */
export function RingOverlay() {
  const { active, preview, dismissed, empty, } = useRing();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (dismissed) {
      setVisible(false);
    } else if (active || preview.length > 0) {
      setVisible(true);
    }
  }, [active, preview, dismissed]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        api.ringDismiss();
        setVisible(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!visible || empty) return null;

  return (
    <div
      className="pointer-events-auto fixed left-1/2 top-4 z-50 -translate-x-1/2 select-none rounded-lg border border-border bg-bg-elevated/95 px-3 py-2 shadow-2xl backdrop-blur"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1 text-[11px] uppercase tracking-wider text-fg-muted">
          <ChevronLeft className="h-3 w-3" />
          <span>ring</span>
          <ChevronRight className="h-3 w-3" />
        </div>
        <div className="flex items-center gap-2">
          {preview.map((slot) => {
            const isActive = active?.id === slot.clip_id;
            return (
              <div
                key={slot.clip_id}
                className={[
                  "max-w-[180px] truncate rounded px-2 py-1 text-xs",
                  isActive
                    ? "bg-accent/20 text-accent ring-1 ring-accent/40"
                    : "bg-bg-overlay/60 text-fg-muted",
                ].join(" ")}
                title={slot.preview}
              >
                {slot.preview || `(${slot.kind})`}
              </div>
            );
          })}
        </div>
        {active && (
          <span className="ml-2 text-[11px] text-fg-subtle">
            [{active.index + 1}/{active.total}]
          </span>
        )}
        <button
          type="button"
          onClick={() => {
            api.ringDismiss();
            setVisible(false);
          }}
          className="ml-1 rounded p-1 text-fg-muted hover:bg-bg-overlay hover:text-fg"
          aria-label="Dismiss ring"
        >
          <X className="h-3 w-3" />
        </button>
      </div>
    </div>
  );
}

/**
 * A small "ring is empty" toast.
 */
export function RingEmptyToast({ visible }: { visible: boolean }) {
  if (!visible) return null;
  return (
    <div className="pointer-events-none fixed left-1/2 top-4 z-50 -translate-x-1/2 select-none rounded-lg border border-border bg-bg-elevated/95 px-3 py-2 text-xs text-fg-muted shadow-2xl backdrop-blur">
      <div className="flex items-center gap-2">
        <CircleAlert className="h-3 w-3" />
        <span>Nothing in the ring yet — copy something first.</span>
      </div>
    </div>
  );
}
