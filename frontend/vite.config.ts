import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

import tauriConfig from "../crates/portolano-app/tauri.conf.json" with { type: "json" };

/** The CSP of the app (SPEC-UI §4.8) as an HTTP header value. */
const appCsp = Object.entries(tauriConfig.app.security.csp)
  .map(([directive, sources]) => `${directive} ${sources}`)
  .join("; ");

// Settings from the Tauri guide for Vite (https://v2.tauri.app/start/frontend/vite/).
export default defineConfig({
  plugins: [react()],
  // Tauri prints its own output in the same terminal.
  clearScreen: false,
  server: {
    // `devUrl` of tauri.conf.json.
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/crates/**", "**/target/**"] },
  },
  // `npm run e2e` serves the built frontend with the CSP of the app, so that
  // a component that needs more than the policy allows fails there first.
  preview: {
    port: 4173,
    strictPort: true,
    headers: { "Content-Security-Policy": appCsp },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // WKWebView of macOS 14, the oldest supported system.
    target: "safari17",
    // Small assets would become `data:` URLs, which `font-src 'self'` refuses.
    assetsInlineLimit: 0,
    minify: !process.env["TAURI_ENV_DEBUG"],
    sourcemap: Boolean(process.env["TAURI_ENV_DEBUG"]),
  },
});
