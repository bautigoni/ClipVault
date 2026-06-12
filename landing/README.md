# ClipVault — landing

Static, single-page marketing site for ClipVault. No backend, no Tauri. Built
with Vite + React + Tailwind. Two deploy targets:

- **GitHub Pages** via `.github/workflows/deploy-landing.yml` (auto on push to `main`).
- **Self-hosted** via the included `Dockerfile` + `docker-compose.yml` (served by Caddy in a container).

## Develop

```bash
cd landing
npm install
npm run dev
```

Open <http://localhost:5173>.

## Build

```bash
npm run build
# outputs to landing/dist/
```

## Deploy to GitHub Pages

Push to `main` — the workflow builds `landing/dist/` and publishes it.

## Deploy to a VPS (Docker + Caddy)

The `Dockerfile` builds a tiny image (Caddy serving `dist/`) and
`docker-compose.yml` starts it on the internal Docker network. The host's
Caddy then reverse-proxies a public hostname to it.

```bash
# On the host (after `cd landing`):
docker compose up -d --build

# Then add to /etc/caddy/Caddyfile on the host:
#   clipvault.bauhub.online {
#       reverse_proxy 127.0.0.1:3008
#   }
# and reload:
sudo systemctl reload caddy
```

`3008` is just an example port — pick any free one and update both
`docker-compose.yml`'s `expose:` and the host Caddyfile to match.

## Editing

All sections live in `src/components/`:

| File | Section |
| --- | --- |
| `Nav.tsx` | Sticky top nav with anchor links + GitHub / Download CTAs |
| `Hero.tsx` | Headline, tagline, primary CTA, static palette preview |
| `Features.tsx` | 9-card feature grid |
| `HowItWorks.tsx` | 3-step install flow |
| `LiveDemo.tsx` | Interactive mini-palette — search, navigate, multi-select, merge |
| `WhyClipVault.tsx` | Privacy / native / open-source pitch |
| `Install.tsx` | Download / build-from-source cards + 4-step instructions |
| `Footer.tsx` | Footer with link group |

The two URLs to update if you fork the repo are the `REPO_URL` and
`INSTALLER_URL` constants in `src/lib/config.ts`.
