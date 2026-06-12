// Single source of truth for the public repo URL. Edit here once and it
// propagates to every CTA, footer link, and install instruction.
export const REPO_URL = "https://github.com/bautigoni/ClipVault";
export const RELEASES_URL = `${REPO_URL}/releases/latest`;
export const ISSUES_URL = `${REPO_URL}/issues`;

// Direct URL to the Windows installer asset.
//
// The default points at the self-hosted mirror on
// `clipvault.bauhub.online` (served by a tiny Python HTTP server
// reverse-proxied through Caddy). That mirror is always reachable even
// before a GitHub release is published. If the direct asset 404s, the
// download buttons fall back to `RELEASES_URL` so the user can pick
// the latest asset manually.
//
// To point at the GitHub release instead of the mirror at build time:
//   VITE_INSTALLER_URL=https://github.com/bautigoni/ClipVault/releases/download/v0.1.0/ClipVault_0.1.0_x64-setup.exe \
//     npm run build
const DEFAULT_INSTALLER_URL =
  "https://clipvault.bauhub.online/downloads/ClipVault_0.1.0_x64-setup.exe";
export const INSTALLER_URL: string =
  (import.meta.env.VITE_INSTALLER_URL as string | undefined)?.trim() ||
  DEFAULT_INSTALLER_URL;
export const INSTALLER_FILENAME = "ClipVault_0.1.0_x64-setup.exe";
