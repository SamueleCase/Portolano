import { Gallery } from "./Gallery";
import { galleryOptions } from "./galleryOptions";
import { GlassPerformance } from "./GlassPerformance";

/** `?gallery`: the catalogue; `?gallery=glass`: the glass measurement. */
export function GalleryRoot() {
  const options = galleryOptions(window.location.search);
  return options.view === "glass" ? <GlassPerformance /> : <Gallery options={options} />;
}
