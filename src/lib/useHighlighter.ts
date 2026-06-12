import { useEffect, useState } from "react";
import { createHighlighter, type Highlighter, type BundledLanguage } from "shiki";

/**
 * Lazy-loaded Shiki highlighter. Bundles only the languages we support so the
 * initial JS payload stays small; the highlighter is created on first use and
 * cached for the rest of the session.
 */
let highlighterPromise: Promise<Highlighter> | null = null;
const SUPPORTED: BundledLanguage[] = [
  "typescript",
  "javascript",
  "python",
  "rust",
  "go",
  "sql",
  "bash",
  "json",
  "yaml",
  "markdown",
];
const FALLBACK_LANG: BundledLanguage = "bash";

function getHighlighter(theme: "github-dark" | "github-light" = "github-dark") {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: [theme],
      langs: SUPPORTED,
    });
  }
  return highlighterPromise;
}

export function useHighlighter(theme: "github-dark" | "github-light" = "github-dark") {
  const [highlighter, setHighlighter] = useState<Highlighter | null>(null);
  useEffect(() => {
    let active = true;
    getHighlighter(theme).then((h) => {
      if (active) setHighlighter(h);
    });
    return () => {
      active = false;
    };
  }, [theme]);
  return highlighter;
}

export function highlight(
  highlighter: Highlighter,
  code: string,
  lang: string,
  theme: "github-dark" | "github-light" = "github-dark"
): string {
  const language: BundledLanguage = SUPPORTED.includes(lang as BundledLanguage)
    ? (lang as BundledLanguage)
    : FALLBACK_LANG;
  return highlighter.codeToHtml(code, { lang: language, theme });
}
