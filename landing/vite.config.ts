import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Static landing, deployable to GitHub Pages / Netlify / etc.
// `base` is the repo name so it works at https://USER.github.io/ClipVault/
export default defineConfig({
  plugins: [react()],
  base: "./",
  build: {
    outDir: "dist",
    assetsDir: "assets",
  },
});
