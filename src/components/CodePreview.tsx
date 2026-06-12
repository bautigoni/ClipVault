import { useEffect, useMemo, useState } from "react";
import { useHighlighter, highlight } from "@/lib/useHighlighter";
import { useTheme } from "@/stores/theme";

interface Props {
  code: string;
  language: string;
  className?: string;
  maxHeight?: number;
}

/**
 * Read-only, syntax-highlighted code block. Falls back to a plain <pre> while
 * Shiki is loading (or if the language isn't supported).
 */
export function CodePreview({ code, language, className, maxHeight = 360 }: Props) {
  const { theme } = useTheme();
  const shikiTheme = theme === "light" ? "github-light" : "github-dark";
  const highlighter = useHighlighter(shikiTheme);
  const [isOverflowing, setIsOverflowing] = useState(false);

  const html = useMemo(() => {
    if (!highlighter) return "";
    try {
      return highlight(highlighter, code, language, shikiTheme);
    } catch {
      return "";
    }
  }, [highlighter, code, language, shikiTheme]);

  useEffect(() => {
    if (!html) return;
    const el = document.createElement("div");
    el.innerHTML = html;
    const pre = el.querySelector("pre");
    if (pre && maxHeight) {
      setIsOverflowing(pre.scrollHeight > maxHeight);
    }
  }, [html, maxHeight]);

  if (!highlighter || !html) {
    return (
      <pre
        className={`overflow-auto rounded-md bg-bg-overlay p-3 font-mono text-xs text-fg ${className ?? ""}`}
        style={{ maxHeight }}
      >
        <code>{code}</code>
      </pre>
    );
  }

  return (
    <div
      className={`relative overflow-auto rounded-md bg-bg-overlay p-3 text-xs ${className ?? ""}`}
      style={{ maxHeight }}
    >
      <div
        className="shiki [&_pre]:!bg-transparent [&_pre]:p-0"
        dangerouslySetInnerHTML={{ __html: html }}
      />
      {isOverflowing && (
        <div className="pointer-events-none absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-bg-overlay to-transparent" />
      )}
    </div>
  );
}
