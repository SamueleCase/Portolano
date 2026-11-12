import "./design/fonts";
import "./design/global.css";

import { QueryClient } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./app/App";
import { applyTokens } from "./design/tokens";

// The end-to-end tests run the frontend alone: the fake backend answers the
// commands in place of Rust. Vite drops this branch from the other builds.
if (import.meta.env.MODE === "e2e") {
  const { installFakeBackend } = await import("./test/fakeBackend");
  installFakeBackend();
}

applyTokens(document.documentElement);
const root = document.getElementById("root");

// The component gallery (`?gallery`) exists only in development and e2e
// builds: Vite drops this branch, and the gallery with it, from the others.
const gallery =
  (import.meta.env.DEV || import.meta.env.MODE === "e2e") &&
  new URLSearchParams(window.location.search).has("gallery");

if (root && gallery) {
  const { GalleryRoot } = await import("./gallery/GalleryRoot");
  createRoot(root).render(
    <StrictMode>
      <GalleryRoot />
    </StrictMode>,
  );
} else if (root) {
  createRoot(root).render(
    <StrictMode>
      <App queryClient={new QueryClient()} />
    </StrictMode>,
  );
}
