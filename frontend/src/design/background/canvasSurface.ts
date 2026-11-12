import type { ConstellationSurface } from "./animator";
import {
  type ConstellationFrame,
  type ConstellationPoint,
  drawConstellation,
  movePoints,
} from "./constellation";

/**
 * The largest bitmap the canvas allocates, 4096 × 4096 pixels. It bounds the
 * memory and the cost of painting up to 30 times per second (DEC-028.6): a
 * window of 1280 × 800 at DPR 2 is a quarter of it, but beyond the limit the
 * scale goes below the DPR. That happens with the screenshot of the whole
 * gallery, and also with a full-screen window on a large HiDPI display (a
 * window of 3008 × 1692 points draws at about 1.82 instead of 2): the lines
 * are then a little softer, never blurred by a CSS upscale of a tiny bitmap.
 */
const MAX_BITMAP_PIXELS = 4096 * 4096;

/** The scale of the bitmap: the DPR, unless the bitmap would pass the limit. */
export function bitmapScale(devicePixelRatio: number, width: number, height: number): number {
  const area = Math.max(1, width * height);
  return Math.min(devicePixelRatio || 1, Math.sqrt(MAX_BITMAP_PIXELS / area));
}

/**
 * Paints `points` on `canvas`, with a bitmap of the canvas size times the
 * device pixel ratio so that the lines stay sharp on Retina displays
 * (inventario §3.1: the prototype ignored the DPR).
 */
export function createCanvasSurface(
  canvas: HTMLCanvasElement,
  context: CanvasRenderingContext2D,
  points: ConstellationPoint[],
): ConstellationSurface {
  const fit = () => {
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    const ratio = bitmapScale(window.devicePixelRatio, width, height);
    const bitmapWidth = Math.round(width * ratio);
    const bitmapHeight = Math.round(height * ratio);
    if (canvas.width !== bitmapWidth || canvas.height !== bitmapHeight) {
      canvas.width = bitmapWidth;
      canvas.height = bitmapHeight;
    }
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    return { width, height };
  };
  return {
    draw(frame: ConstellationFrame) {
      const { width, height } = fit();
      movePoints(points, frame.step, width, height);
      drawConstellation(context, points, width, height, frame);
    },
    clear() {
      const { width, height } = fit();
      context.clearRect(0, 0, width, height);
    },
  };
}
