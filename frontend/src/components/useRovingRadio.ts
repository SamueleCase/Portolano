import { type KeyboardEvent, useRef } from "react";

interface RovingRadioOptions {
  /** How many options the group has. */
  count: number;
  /** The index of the chosen option, or -1. */
  selected: number;
  /** Options shown but not choosable are skipped. */
  available?: (index: number) => boolean;
  /** A grid of this many columns: the arrows move in two dimensions, without wrapping. */
  columns?: number;
  /** Chooses the option at `index` (the arrows move and choose, as radios do). */
  onMove: (index: number) => void;
}

const alwaysAvailable = () => true;

function nextInLine(
  index: number,
  step: number,
  count: number,
  available: (index: number) => boolean,
): number | undefined {
  for (let offset = 1; offset <= count; offset++) {
    const candidate = (((index + step * offset) % count) + count) % count;
    if (available(candidate)) return candidate;
  }
  return undefined;
}

/**
 * The keyboard of a radio group (WAI-ARIA): one tab stop, on the chosen
 * option or else on the first available one, and the arrows choose the next
 * available option. Shared by the segmented control, the credential modes,
 * the colors and the icon grid.
 */
export function useRovingRadio({
  count,
  selected,
  available = alwaysAvailable,
  columns,
  onMove,
}: RovingRadioOptions) {
  const elements = useRef(new Map<number, HTMLElement>());
  const firstAvailable = Array.from({ length: count }, (_, index) => index).find(available);
  const focusable =
    selected >= 0 && selected < count && available(selected) ? selected : firstAvailable;

  const target = (key: string, index: number): number | undefined => {
    if (columns !== undefined) {
      const step = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: columns, ArrowUp: -columns }[key];
      if (step === undefined) return undefined;
      const candidate = index + step;
      return candidate >= 0 && candidate < count && available(candidate) ? candidate : undefined;
    }
    if (key === "Home") return firstAvailable;
    if (key === "End") return nextInLine(0, -1, count, available);
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[key];
    return step === undefined ? undefined : nextInLine(index, step, count, available);
  };

  return {
    /** The `tabIndex` of the option at `index`. */
    tabIndexOf: (index: number) => (index === focusable ? 0 : -1),
    /** The `ref` of the option at `index`, to move the focus with the choice. */
    register: (index: number) => (element: HTMLElement | null) => {
      if (element) elements.current.set(index, element);
      else elements.current.delete(index);
    },
    /** The key handler of the option at `index`. */
    keyDown: (event: KeyboardEvent, index: number) => {
      const next = target(event.key, index);
      if (next === undefined || next === index) return;
      event.preventDefault();
      onMove(next);
      elements.current.get(next)?.focus();
    },
  };
}
