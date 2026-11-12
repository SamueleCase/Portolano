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
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App queryClient={new QueryClient()} />
    </StrictMode>,
  );
}
