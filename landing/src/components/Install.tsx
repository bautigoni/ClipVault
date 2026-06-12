import { Download, Github, Check, Copy, Terminal, ExternalLink } from "lucide-react";
import { useState } from "react";
import { REPO_URL, RELEASES_URL, INSTALLER_URL, INSTALLER_FILENAME } from "../lib/config";

const GITHUB_URL = REPO_URL;

/**
 * A download button that uses the configured direct installer URL. If the
 * user already installed a previous version, the browser will start the
 * download immediately. If the asset 404s (e.g. a tagged release hasn't
 * been published yet), we fall back to the GitHub releases page so the
 * user can grab whatever's there.
 */
function DownloadInstallerLink({
  className,
  primary = false,
  children,
}: {
  className?: string;
  primary?: boolean;
  children: React.ReactNode;
}) {
  const [failed, setFailed] = useState(false);
  const href = failed ? RELEASES_URL : INSTALLER_URL;
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      onClick={() => {
        // If the GitHub release 404s, the download won't start — open the
        // releases page in a new tab so the user always lands somewhere
        // useful. The other button (and the link's `href`) fall back
        // gracefully on the next click as well.
        // We optimistically flip `failed` after a short delay; if the
        // browser blocked the navigation (e.g. popup blocker), the
        // releases tab is still a better landing than a dead `data:`.
        window.setTimeout(() => setFailed(true), 1500);
      }}
      className={className}
    >
      {children}
      {failed && (
        <ExternalLink className="ml-1 inline h-3.5 w-3.5 align-text-bottom opacity-70" />
      )}
    </a>
  );
}

export function Install() {
  return (
    <section id="install" className="relative py-20 sm:py-28">
      <div className="container-page">
        <div className="mx-auto max-w-3xl">
          <div className="text-center">
            <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-accent">
              Get it
            </p>
            <h2 className="text-balance text-3xl font-bold tracking-tight sm:text-5xl">
              Install in 30 seconds.
            </h2>
            <p className="mt-4 text-pretty text-base text-fg-muted sm:text-lg">
              Windows 10/11 · x64 · ~5 MB
            </p>
          </div>

          <div className="mt-12 grid gap-4 sm:grid-cols-2">
            <DownloadInstallerLink className="card group flex flex-col items-start hover:border-accent/50">
              <div className="mb-3 inline-flex h-10 w-10 items-center justify-center rounded-lg bg-accent text-white shadow-lg shadow-accent/30">
                <Download className="h-5 w-5" />
              </div>
              <h3 className="text-base font-semibold">Download installer</h3>
              <p className="mt-1 text-sm text-fg-muted">
                Recommended. NSIS wizard, Start menu shortcut, autostart optional.
              </p>
              <div className="mt-4 flex items-center gap-2 text-xs text-fg-muted">
                <Check className="h-3.5 w-3.5 text-success" />
                <span>{INSTALLER_FILENAME}</span>
              </div>
              <div className="mt-3 self-end text-sm font-semibold text-accent transition-transform duration-200 group-hover:translate-x-1">
                Download now →
              </div>
            </DownloadInstallerLink>

            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noreferrer"
              className="card group flex flex-col items-start hover:border-fg-muted/60"
            >
              <div className="mb-3 inline-flex h-10 w-10 items-center justify-center rounded-lg bg-bg-overlay text-fg">
                <Github className="h-5 w-5" />
              </div>
              <h3 className="text-base font-semibold">Build from source</h3>
              <p className="mt-1 text-sm text-fg-muted">
                For the curious. Requires Rust + Node. About 10 minutes on a clean machine.
              </p>
              <CodeSnippet
                command={`git clone ${REPO_URL} && cd ClipVault && npm install && npm run tauri:build`}
                className="mt-4"
              />
              <div className="mt-3 self-end text-sm font-semibold text-fg-muted transition-transform duration-200 group-hover:translate-x-1">
                Open repo →
              </div>
            </a>
          </div>

          <ol className="mt-12 space-y-3 text-sm text-fg-muted">
            <Step n={1}>
              Download <span className="font-mono text-fg">{INSTALLER_FILENAME}</span>{" "}
              from the latest release.
            </Step>
            <Step n={2}>
              Double-click the installer. NSIS will guide you. Per-user install is
              supported — no admin needed.
            </Step>
            <Step n={3}>
              Launch ClipVault from the Start menu. It lives in the system tray. Hit{" "}
              <span className="kbd">Ctrl</span>+<span className="kbd">Shift</span>+
              <span className="kbd">V</span> anywhere to open the palette.
            </Step>
            <Step n={4}>
              Optional: open <span className="font-mono text-fg">Settings</span> to
              change the hotkey, theme, merge separator, and Ctrl+↑/↓ jump size.
            </Step>
          </ol>

          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <DownloadInstallerLink primary className="btn-primary">
              <Download className="h-4 w-4" />
              Download for Windows
            </DownloadInstallerLink>
            <a href={GITHUB_URL} target="_blank" rel="noreferrer" className="btn-ghost">
              <Github className="h-4 w-4" />
              Star on GitHub
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}

function Step({ n, children }: { n: number; children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-3 rounded-lg border border-border bg-bg-elevated/40 p-4">
      <span className="grid h-6 w-6 shrink-0 place-items-center rounded-full bg-bg-overlay text-[11px] font-bold text-fg">
        {n}
      </span>
      <div className="pt-0.5">{children}</div>
    </li>
  );
}

function CodeSnippet({
  command,
  className = "",
}: {
  command: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* ignore */
    }
  };
  return (
    <div
      className={`group flex items-center gap-2 overflow-hidden rounded-md border border-border bg-bg/80 font-mono text-[11px] ${className}`}
    >
      <span className="flex h-7 items-center gap-1.5 border-r border-border bg-bg-overlay px-2 text-fg-muted">
        <Terminal className="h-3 w-3" />
      </span>
      <code className="flex-1 truncate px-2 py-1.5 text-fg-muted">{command}</code>
      <button
        type="button"
        onClick={copy}
        aria-label="Copy command"
        className="flex h-7 w-7 items-center justify-center border-l border-border text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-success" />
        ) : (
          <Copy className="h-3.5 w-3.5" />
        )}
      </button>
    </div>
  );
}
