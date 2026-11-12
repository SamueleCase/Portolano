import type { BackgroundMode } from "../design/background/backgroundControl";
import type { Density } from "../design/appearance";

/** How the gallery starts, from its URL: `?gallery&density=compact&motion=reduced`. */
export interface GalleryOptions {
  /** `?gallery=glass`: the page of the glass measurement instead of the catalogue. */
  view: "catalogue" | "glass";
  density: Density;
  reduceMotion: boolean;
  background: BackgroundMode;
}

const backgrounds: readonly BackgroundMode[] = ["on", "reduced", "off"];

/** Reads the options of the gallery from the query string. */
export function galleryOptions(search: string): GalleryOptions {
  const params = new URLSearchParams(search);
  const background = backgrounds.find((mode) => mode === params.get("background")) ?? "on";
  return {
    view: params.get("gallery") === "glass" ? "glass" : "catalogue",
    density: params.get("density") === "compact" ? "compact" : "normal",
    reduceMotion: params.get("motion") === "reduced",
    background,
  };
}

/** The sky of the gallery is always the same, so that screenshots compare. */
export const GALLERY_SEED = 2026;
