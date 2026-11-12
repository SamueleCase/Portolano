import "@testing-library/jest-dom/vitest";

import { cleanup } from "@testing-library/react";
import { clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, vi } from "vitest";

afterEach(() => {
  cleanup();
  clearMocks();
});

// The first test of a file also pays for jsdom and for loading the
// components: on a busy machine that alone passes the 5 s default.
vi.setConfig({ testTimeout: 30_000 });

// jsdom has no layout: what the components observe never changes size.
class StillResizeObserver implements ResizeObserver {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = StillResizeObserver;
}

// jsdom has no font loading: the fonts are always ready.
if (!("fonts" in document)) {
  Object.defineProperty(document, "fonts", { value: { ready: Promise.resolve() } });
}
