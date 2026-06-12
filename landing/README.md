# ClipVault — landing

Static, single-page marketing site for ClipVault. No backend, no Tauri. Built
with Vite + React + Tailwind. Deploys to GitHub Pages via
`.github/workflows/deploy-landing.yml`.

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

## Deploy

Push to `main` — the GitHub Action builds and publishes the `landing/dist/`
folder to GitHub Pages at the repo's Pages URL (typically
`https://<owner>.github.io/<repo>/`).

To use a different base path, edit `base` in `vite.config.ts`.

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

The two URLs to update if you fork the repo are the `GITHUB_URL` and
`INSTALLER_URL` constants at the top of `Nav.tsx`, `Hero.tsx`, and
`Install.tsx`.
