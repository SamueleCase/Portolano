import { useSyncExternalStore } from "react";

/** The density of the interface (ambiguity 8). */
export type Density = "normal" | "compact";

/** The appearance preferences that change the whole window (SPEC-UI §5.2). */
export interface Appearance {
  density: Density;
  /** The "Riduci movimento" preference; the system setting applies as well. */
  reduceMotion: boolean;
}

/**
 * Applies `appearance` to the root element: the token stylesheet reads
 * `data-density` and `data-motion` (see `tokens.ts`).
 */
export function applyAppearance(root: HTMLElement, appearance: Appearance): void {
  root.dataset["density"] = appearance.density;
  if (appearance.reduceMotion) {
    root.dataset["motion"] = "reduced";
  } else {
    delete root.dataset["motion"];
  }
}

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function reducedMotionQuery(): MediaQueryList | undefined {
  // jsdom and some test environments have no `matchMedia`.
  return typeof window.matchMedia === "function"
    ? window.matchMedia(REDUCED_MOTION_QUERY)
    : undefined;
}

/** Whether the system asks for reduced motion. */
export function systemPrefersReducedMotion(): boolean {
  return reducedMotionQuery()?.matches ?? false;
}

function subscribeToSystemMotion(onChange: () => void): () => void {
  const query = reducedMotionQuery();
  query?.addEventListener("change", onChange);
  return () => query?.removeEventListener("change", onChange);
}

/** Whether motion is reduced: by the preference or by the system (inventario §3.5). */
export function useReducedMotion(preference: boolean): boolean {
  const system = useSyncExternalStore(subscribeToSystemMotion, systemPrefersReducedMotion);
  return preference || system;
}
