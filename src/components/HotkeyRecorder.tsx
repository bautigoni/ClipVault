import { useEffect, useState } from "react";
import { Keyboard } from "lucide-react";

interface Props {
  value: string;
  onChange: (combo: string) => void;
  className?: string;
}

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);
const IGNORED_KEYS = new Set([
  "Tab",
  "CapsLock",
  "NumLock",
  "ScrollLock",
  "Dead",
  "Unidentified",
  "Process",
  "AltGraph",
]);

/** Keys that should cancel recording without setting a hotkey. */
const CANCEL_KEYS = new Set(["Escape"]);

function formatCombo(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Meta");

  const key = e.key;
  if (MODIFIER_KEYS.has(key) || IGNORED_KEYS.has(key)) {
    return null;
  }

  let main = key;
  if (key === " ") main = "Space";
  // Capitalize single letters and digits
  if (main.length === 1) {
    main = main.toUpperCase();
  } else {
    main = main.charAt(0).toUpperCase() + main.slice(1);
  }
  parts.push(main);
  return parts.join("+");
}

/**
 * Click to start recording, then press the desired key combination. The combo
 * is emitted as a string compatible with `tauri-plugin-global-shortcut`
 * (e.g. `Ctrl+Shift+V`).
 */
export function HotkeyRecorder({ value, onChange, className }: Props) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (CANCEL_KEYS.has(e.key)) {
        setRecording(false);
        return;
      }
      const combo = formatCombo(e);
      if (combo) {
        onChange(combo);
        setRecording(false);
      }
    };
    const onBlur = () => setRecording(false);
    window.addEventListener("keydown", onKey, { capture: true });
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("keydown", onKey, { capture: true });
      window.removeEventListener("blur", onBlur);
    };
  }, [recording, onChange]);

  return (
    <button
      type="button"
      onClick={() => setRecording(true)}
      data-recording={recording}
      className={`inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm transition-colors ${
        recording
          ? "border-accent bg-accent/15 text-accent animate-pulse-soft"
          : "border-border bg-bg text-fg hover:border-accent/50"
      } ${className ?? ""}`}
    >
      <Keyboard className="h-4 w-4" />
      {recording ? (
        <span>Press a key combination…</span>
      ) : (
        <>
          <span className="font-mono">{value || "Not set"}</span>
          {value && (
            <span
              className="ml-1 cursor-pointer text-fg-muted hover:text-danger"
              onClick={(e) => {
                e.stopPropagation();
                onChange("");
              }}
              title="Clear"
            >
              ×
            </span>
          )}
        </>
      )}
    </button>
  );
}

export { formatCombo };
