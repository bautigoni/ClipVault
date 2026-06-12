export const SUPPORTED_LANGUAGES = [
  "typescript",
  "javascript",
  "python",
  "rust",
  "go",
  "sql",
  "bash",
  "powershell",
  "json",
  "yaml",
  "markdown",
  "plaintext",
] as const;

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

/** Languages that Shiki will syntax-highlight. Falls back to bash for anything else. */
export const SHIKI_LANGUAGES: ReadonlySet<string> = new Set([
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
]);
