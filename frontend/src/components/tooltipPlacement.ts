/** Where a tooltip goes, in viewport pixels: under its trigger, or above it. */
export type TooltipPlacement =
  { above: false; left: number; top: number } | { above: true; left: number; bottom: number };

interface Size {
  width: number;
  height: number;
}

interface TriggerBox extends Size {
  top: number;
  bottom: number;
  left: number;
}

/**
 * The place of a tooltip of `size` for `trigger`, `gap` away from it: under
 * it and centered, as usual; above it when the window has no room below
 * (and has it above); moved sideways to stay inside the window.
 */
export function placeTooltip(
  trigger: TriggerBox,
  size: Size,
  gap: number,
  viewport: Size,
): TooltipPlacement {
  const centered = trigger.left + trigger.width / 2 - size.width / 2;
  const left = Math.max(0, Math.min(centered, viewport.width - size.width));
  const fitsBelow = trigger.bottom + gap + size.height <= viewport.height;
  const fitsAbove = trigger.top - gap - size.height >= 0;
  if (fitsBelow || !fitsAbove) return { above: false, left, top: trigger.bottom };
  return { above: true, left, bottom: viewport.height - trigger.top };
}
