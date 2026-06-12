// Single source of truth for the public repo URL. Edit here once and it
// propagates to every CTA, footer link, and install instruction.
export const REPO_URL = "https://github.com/bautigoni/ClipVault";
export const RELEASES_URL = `${REPO_URL}/releases/latest`;
export const ISSUES_URL = `${REPO_URL}/issues`;

// Direct URL to the Windows installer asset. Bypasses the GitHub release
// redirect so the browser starts the download immediately on click.
// Pinned to v0.1.0 — bump alongside `version` in tauri.conf.json when
// shipping a new release, or move to a "latest" symlink on a CDN.
export const INSTALLER_URL =
  "https://github.com/bautigoni/ClipVault/releases/download/v0.1.0/ClipVault_0.1.0_x64-setup.exe";
export const INSTALLER_FILENAME = "ClipVault_0.1.0_x64-setup.exe";
