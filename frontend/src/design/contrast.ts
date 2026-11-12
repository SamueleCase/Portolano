/** A color as sRGB channels (0–255) with an alpha (0–1). */
export interface Rgba {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

const HEX = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i;
const RGBA = /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+)\s*)?\)$/;

/** Reads a token color: `#RRGGBB`, `rgb(r,g,b)` or `rgba(r,g,b,a)`. */
export function parseColor(value: string): Rgba {
  const hex = HEX.exec(value);
  if (hex) {
    const [red, green, blue] = hex.slice(1, 4).map((channel) => Number.parseInt(channel, 16));
    return { red: red ?? 0, green: green ?? 0, blue: blue ?? 0, alpha: 1 };
  }
  const rgba = RGBA.exec(value);
  if (rgba) {
    const [red, green, blue, alpha] = rgba.slice(1, 5).map(Number);
    return {
      red: red ?? 0,
      green: green ?? 0,
      blue: blue ?? 0,
      alpha: alpha === undefined || Number.isNaN(alpha) ? 1 : alpha,
    };
  }
  throw new Error(`unknown color: ${value}`);
}

/**
 * The opaque color seen through `layers`, listed from the bottom up: the
 * glass and the tints of the palette are translucent, so a text sits on the
 * blend of every surface below it. The bottom layer must be opaque.
 */
export function composite(layers: readonly string[]): Rgba {
  const [bottom, ...above] = layers.map(parseColor);
  if (!bottom || bottom.alpha !== 1) {
    throw new Error("the bottom layer must be opaque");
  }
  return above.reduce(
    (below, layer) => ({
      red: layer.red * layer.alpha + below.red * (1 - layer.alpha),
      green: layer.green * layer.alpha + below.green * (1 - layer.alpha),
      blue: layer.blue * layer.alpha + below.blue * (1 - layer.alpha),
      alpha: 1,
    }),
    bottom,
  );
}

function linear(channel: number): number {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

/** The relative luminance of an opaque color (WCAG 2.2). */
export function luminance(color: Rgba): number {
  return 0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue);
}

/** The WCAG contrast ratio of two opaque colors, from 1 to 21. */
export function contrastRatio(first: Rgba, second: Rgba): number {
  const [lighter, darker] = [luminance(first), luminance(second)].sort((a, b) => b - a);
  return ((lighter ?? 0) + 0.05) / ((darker ?? 0) + 0.05);
}
