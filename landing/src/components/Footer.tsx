import { Github } from "lucide-react";
import { REPO_URL, ISSUES_URL } from "../lib/config";

const GITHUB_URL = REPO_URL;

export function Footer() {
  return (
    <footer className="border-t border-border bg-bg-elevated/30">
      <div className="container-page flex flex-col items-start gap-6 py-10 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3">
          <span className="grid h-8 w-8 place-items-center rounded-md bg-accent/20 text-accent">
            <Github className="h-4 w-4" />
          </span>
          <div>
            <p className="text-sm font-semibold">ClipVault</p>
            <p className="text-xs text-fg-muted">MIT licensed · 100% local</p>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-fg-muted">
          <a href="#features" className="hover:text-fg">
            Features
          </a>
          <a href="#demo" className="hover:text-fg">
            Preview
          </a>
          <a href="#install" className="hover:text-fg">
            Install
          </a>
          <a
            href={ISSUES_URL}
            target="_blank"
            rel="noreferrer"
            className="hover:text-fg"
          >
            Report an issue
          </a>
        </div>
      </div>
      <div className="border-t border-border/60">
        <p className="container-page py-4 text-center text-[11px] text-fg-subtle">
          Built with Tauri, Rust, React, and a lot of Ctrl+V. No telemetry. No cloud. No
          nonsense.
        </p>
      </div>
    </footer>
  );
}
