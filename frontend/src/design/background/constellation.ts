import { constellationColors } from "../tokens";

/** A point of the constellation, in coordinates normalised to the canvas (0–1). */
export interface ConstellationPoint {
  x: number;
  y: number;
  /** Speed in CSS pixels per second. */
  vx: number;
  vy: number;
  big: boolean;
}

/** What one frame draws (inventario §3.1). */
export interface ConstellationFrame {
  /** The alpha multiplier of lines, points and rings. */
  alpha: number;
  /** How far the points move, in seconds of their speed. */
  step: number;
  /** The animation time, which drives the rings. */
  time: number;
}

const POINT_COUNT = 120;
const MAX_SPEED = 26;
const BIG_SHARE = 0.12;
const LINK_DISTANCE_SQUARED = 170 * 170;
const LINE_ALPHA = 0.34;
const POINT_ALPHA = 0.75;
const RING_ALPHA = 0.3;
const POINT_RADIUS = 1.7;
const BIG_POINT_RADIUS = 3;
const FULL_TURN = Math.PI * 2;

/** A small seeded generator (mulberry32), so that a test can fix the sky. */
export function seededRandom(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let value = state;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}

/** The 120 points of the sky, 12% of them big (inventario §3.1). */
export function createPoints(random: () => number): ConstellationPoint[] {
  return Array.from({ length: POINT_COUNT }, () => ({
    x: random(),
    y: random(),
    vx: (random() - 0.5) * MAX_SPEED,
    vy: (random() - 0.5) * MAX_SPEED,
    big: random() < BIG_SHARE,
  }));
}

function wrap(value: number): number {
  return value - Math.floor(value);
}

/** Moves every point by `step` seconds, wrapping at the edges. */
export function movePoints(
  points: ConstellationPoint[],
  step: number,
  width: number,
  height: number,
): void {
  if (width === 0 || height === 0) {
    return;
  }
  for (const point of points) {
    point.x = wrap(point.x + (point.vx * step) / width);
    point.y = wrap(point.y + (point.vy * step) / height);
  }
}

/** Draws the constellation in CSS pixels on a context already scaled for the DPR. */
export function drawConstellation(
  context: CanvasRenderingContext2D,
  points: readonly ConstellationPoint[],
  width: number,
  height: number,
  frame: ConstellationFrame,
): void {
  context.clearRect(0, 0, width, height);
  context.lineWidth = 1;
  for (let i = 0; i < points.length; i++) {
    const from = points[i];
    for (let j = i + 1; j < points.length && from; j++) {
      const to = points[j];
      if (!to) continue;
      const dx = (from.x - to.x) * width;
      const dy = (from.y - to.y) * height;
      const distance = dx * dx + dy * dy;
      if (distance >= LINK_DISTANCE_SQUARED) continue;
      const alpha = (1 - distance / LINK_DISTANCE_SQUARED) * LINE_ALPHA * frame.alpha;
      context.strokeStyle = `rgba(${constellationColors.line},${alpha})`;
      context.beginPath();
      context.moveTo(from.x * width, from.y * height);
      context.lineTo(to.x * width, to.y * height);
      context.stroke();
    }
  }
  points.forEach((point, index) => {
    const x = point.x * width;
    const y = point.y * height;
    context.fillStyle = `rgba(${constellationColors.point},${POINT_ALPHA * frame.alpha})`;
    context.beginPath();
    context.arc(x, y, point.big ? BIG_POINT_RADIUS : POINT_RADIUS, 0, FULL_TURN);
    context.fill();
    if (point.big) {
      const radius = 7 + 5 * (0.5 + 0.5 * Math.sin(frame.time * 1.6 + index));
      context.strokeStyle = `rgba(${constellationColors.point},${RING_ALPHA * frame.alpha})`;
      context.beginPath();
      context.arc(x, y, radius, 0, FULL_TURN);
      context.stroke();
    }
  });
}
