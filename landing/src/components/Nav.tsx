import { useEffect, useState } from "react";
import { Github, Download, Menu, X, ExternalLink } from "lucide-react";
import { REPO_URL, RELEASES_URL, INSTALLER_URL } from "../lib/config";

const GITHUB_URL = REPO_URL;

/**
 * Same fallback strategy as in `Install.tsx` — if the direct installer
 * link 404s (e.g. no release published yet for this commit), fall back
 * to the GitHub releases page so the user can pick an asset manually.
 */
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

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const links = [
    { href: "#features", label: "Features" },
    { href: "#demo", label: "Preview" },
    { href: "#install", label: "Install" },
  ];

  return (
    <header
      className={`fixed inset-x-0 top-0 z-50 transition-all duration-300 ${
        scrolled
          ? "border-b border-border bg-bg/80 backdrop-blur-xl"
          : "bg-transparent"
      }`}
    >
      <div className="container-page flex h-16 items-center justify-between">
        <a href="#top" className="group flex items-center gap-2.5">
          <LogoMark className="h-7 w-7 transition-transform duration-300 group-hover:scale-110 group-hover:rotate-3" />
          <span className="text-base font-bold tracking-tight">
            Clip<span className="gradient-text-indigo">Vault</span>
          </span>
        </a>

        <nav className="hidden items-center gap-1 md:flex">
          {links.map((l) => (
            <a
              key={l.href}
              href={l.href}
              className="rounded-md px-3 py-1.5 text-sm font-medium text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
            >
              {l.label}
            </a>
          ))}
        </nav>

        <div className="hidden items-center gap-2 md:flex">
          <a
            href={GITHUB_URL}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
            aria-label="GitHub repository"
          >
            <Github className="h-4 w-4" />
            <span>Star</span>
          </a>
          <DownloadInstallerLink className="btn-primary">
            <Download className="h-4 w-4" />
            Download
          </DownloadInstallerLink>
        </div>

        <button
          type="button"
          aria-label="Toggle menu"
          onClick={() => setOpen((v) => !v)}
          className="rounded-md p-2 text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg md:hidden"
        >
          {open ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
        </button>
      </div>

      {open && (
        <div className="border-t border-border bg-bg/95 backdrop-blur-xl md:hidden">
          <div className="container-page flex flex-col gap-1 py-3">
            {links.map((l) => (
              <a
                key={l.href}
                href={l.href}
                onClick={() => setOpen(false)}
                className="rounded-md px-3 py-2 text-sm font-medium text-fg-muted transition-colors hover:bg-bg-overlay hover:text-fg"
              >
                {l.label}
              </a>
            ))}
            <div className="mt-2 flex flex-col gap-2 border-t border-border pt-3">
              <a
                href={GITHUB_URL}
                target="_blank"
                rel="noreferrer"
                className="btn-ghost w-full"
              >
                <Github className="h-4 w-4" />
                GitHub
              </a>
              <DownloadInstallerLink className="btn-primary w-full">
                <Download className="h-4 w-4" />
                Download for Windows
              </DownloadInstallerLink>
            </div>
          </div>
        </div>
      )}
    </header>
  );
}

function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" fill="none" className={className} aria-hidden>
      <rect
        x="8"
        y="6"
        width="48"
        height="52"
        rx="10"
        fill="rgb(24,24,27)"
        stroke="rgb(99,102,241)"
        strokeWidth="3"
      />
      <rect x="20" y="2" width="24" height="10" rx="4" fill="rgb(99,102,241)" />
      <path
        d="M20 24h24M20 32h24M20 40h16"
        stroke="rgb(161,161,170)"
        strokeWidth="3"
        strokeLinecap="round"
      />
    </svg>
  );
}
