import type { ConstellationSurface } from "./animator";
import {
  type ConstellationFrame,
  type ConstellationPoint,
  drawConstellation,
  movePoints,
} from "./constellation";

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
    const ratio = window.devicePixelRatio || 1;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
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
