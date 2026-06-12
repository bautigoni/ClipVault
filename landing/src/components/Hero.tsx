import { useState } from "react";
import { Download, Github, Sparkles, Zap, Lock, ChevronRight, ExternalLink } from "lucide-react";
import { REPO_URL, RELEASES_URL, INSTALLER_URL } from "../lib/config";

const GITHUB_URL = REPO_URL;

function DownloadInstallerLink({
  className,
  children,
}: {
  className?: string;
  children: React.ReactNode;
}) {
  const [failed, setFailed] = useState(false);
  const href = failed ? RELEASES_URL : INSTALLER_URL;
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      onClick={() => window.setTimeout(() => setFailed(true), 1500)}
      className={className}
    >
      {children}
      {failed && (
        <ExternalLink className="ml-1 inline h-3.5 w-3.5 align-text-bottom opacity-70" />
      )}
    </a>
  );
}

export function Hero() {
  return (
    <section id="top" className="relative overflow-hidden pt-32 pb-20 sm:pt-40 sm:pb-28">
      <div className="absolute inset-0 -z-10 glow-bg" />
      <div className="absolute inset-0 -z-10 grid-bg" />

      <div className="container-page">
        <div className="mx-auto max-w-3xl text-center">
          <div
            className="mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-bg-elevated/60 px-3 py-1 text-xs font-medium text-fg-muted backdrop-blur animate-fade-in"
            style={{ animationDelay: "0.05s", animationFillMode: "both" }}
          >
            <Sparkles className="h-3.5 w-3.5 text-accent" />
            <span>Privacy-first &middot; Local-only &middot; 100% on your machine</span>
          </div>

          <h1
            className="text-balance text-4xl font-extrabold tracking-tight sm:text-6xl md:text-7xl animate-fade-in-up"
            style={{ animationDelay: "0.1s", animationFillMode: "both" }}
          >
            Your clipboard,
            <br />
            <span className="gradient-text">but searchable.</span>
          </h1>

          <p
            className="mx-auto mt-6 max-w-2xl text-pretty text-base text-fg-muted sm:text-lg animate-fade-in-up"
            style={{ animationDelay: "0.2s", animationFillMode: "both" }}
          >
            ClipVault captures everything you copy, indexes it instantly, and brings it
            back with a single keystroke. Native Windows app, zero cloud, zero telemetry.
          </p>

          <div
            className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row animate-fade-in-up"
            style={{ animationDelay: "0.3s", animationFillMode: "both" }}
          >
            <DownloadInstallerLink className="btn-primary animate-pulse-glow">
              <Download className="h-4 w-4" />
              Download for Windows
              <ChevronRight className="h-4 w-4" />
            </DownloadInstallerLink>
            <a href={GITHUB_URL} target="_blank" rel="noreferrer" className="btn-ghost">
              <Github className="h-4 w-4" />
              View on GitHub
            </a>
          </div>

          <div
            className="mt-10 flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-xs text-fg-subtle animate-fade-in"
            style={{ animationDelay: "0.45s", animationFillMode: "both" }}
          >
            <span className="flex items-center gap-1.5">
              <Lock className="h-3.5 w-3.5" /> Local-only
            </span>
            <span className="flex items-center gap-1.5">
              <Zap className="h-3.5 w-3.5" /> Instant search
            </span>
            <span className="flex items-center gap-1.5">
              <span className="font-mono">Ctrl+Shift+V</span> to open
            </span>
          </div>
        </div>

        <div
          className="relative mx-auto mt-16 max-w-4xl animate-fade-in-up sm:mt-20"
          style={{ animationDelay: "0.5s", animationFillMode: "both" }}
        >
          <PalettePreview />
        </div>
      </div>
    </section>
  );
}

const SAMPLE_CLIPS = [
  { icon: "file", app: "Chrome", text: "https://github.com/tauri-apps/tauri" },
  { icon: "file", app: "VSCode", text: "useEffect(() => { /* mount */ }, []);" },
  { icon: "link", app: "Slack", text: "deploy logs from staging — all green" },
  { icon: "file", app: "Cursor", text: "SELECT id, hash FROM clips WHERE id = ?" },
  { icon: "file", app: "Notepad", text: "shopping list: milk, eggs, coffee" },
];

function PalettePreview() {
  return (
    <div className="relative">
      <div className="absolute -inset-4 -z-10 rounded-3xl bg-accent/20 blur-3xl" />
      <div className="overflow-hidden rounded-2xl border border-border bg-bg-elevated/90 shadow-2xl shadow-black/50 backdrop-blur">
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <div className="h-3 w-3 rounded-full bg-danger/80" />
          <div className="h-3 w-3 rounded-full bg-warning/80" />
          <div className="h-3 w-3 rounded-full bg-success/80" />
          <div className="ml-2 h-2 w-2 animate-blink rounded-full bg-accent" />
          <span className="ml-2 font-mono text-[11px] text-fg-subtle">
            ClipVault Quick Paste
          </span>
        </div>
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <span className="text-fg-subtle">⌕</span>
          <span className="font-mono text-sm text-fg-muted">Search clips, URLs, code…</span>
          <span className="ml-auto kbd">esc</span>
        </div>
        <ul className="divide-y divide-border/50 p-1">
          {SAMPLE_CLIPS.map((c, i) => (
            <li
              key={i}
              className={`flex items-center gap-3 rounded-md px-3 py-2.5 transition-colors ${
                i === 1 ? "bg-accent/15 ring-1 ring-accent/40" : ""
              }`}
            >
              <span className="grid h-7 w-7 place-items-center rounded-md bg-bg-overlay text-fg-muted">
                {c.icon === "link" ? "🔗" : "📄"}
              </span>
              <div className="min-w-0 flex-1">
                <div className="truncate font-mono text-sm text-fg">{c.text}</div>
                <div className="text-[11px] text-fg-muted">{c.app}</div>
              </div>
              {i === 1 && <span className="kbd">↵</span>}
            </li>
          ))}
        </ul>
        <div className="flex items-center justify-between border-t border-border px-4 py-2 text-[11px] text-fg-muted">
          <div className="flex items-center gap-2">
            <span className="kbd">↑↓</span>
            <span>navigate</span>
            <span className="kbd">␣</span>
            <span>select</span>
            <span className="kbd">↵</span>
            <span>paste</span>
          </div>
          <span>32 results · 0ms</span>
        </div>
      </div>
    </div>
  );
}
