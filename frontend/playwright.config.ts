import { defineConfig, devices } from "@playwright/test";

/** The preview server of `vite.config.ts`. */
const baseURL = "http://localhost:4173";

// End-to-end tests of the frontend alone, in WebKit (the engine of WKWebView),
// against an in-memory fake backend: the build in `e2e` mode installs it in
// place of the IPC of Tauri (SPEC-UI §7).
export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env["CI"]),
  reporter: "list",
  use: { baseURL },
  projects: [
    {
      name: "webkit",
      // The size of the main window (SPEC-UI §4.9).
      use: { ...devices["Desktop Safari"], viewport: { width: 1280, height: 800 } },
    },
  ],
  webServer: {
    command: "vite build --mode e2e --outDir dist-e2e && vite preview --outDir dist-e2e",
    url: baseURL,
    reuseExistingServer: false,
  },
});
