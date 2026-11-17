import type { Page } from "@playwright/test";

/**
 * WebKit, like Safari, moves with Tab only among text fields unless the user
 * turns on keyboard navigation; Option+Tab moves among every control
 * (DEC-032.4). The app itself reaches every control with Tab (DEC-036.1).
 */
export const TAB = "Alt+Tab";
export const BACK_TAB = "Alt+Shift+Tab";

/** The focused element, as a stop of a sweep with Tab. */
export interface FocusStop {
  /** The same for the same element across the sweep. */
  id: string;
  /** The accessible name, or the text, of the element. */
  name: string;
  /** The landmark around it: the toolbar, the tree, a dialog… (`AREAS`). */
  area: string;
  /** Whether its focus shows: a ring, or an accent border or fill. */
  visible: boolean;
}

/** The areas of the main window a stop can be in, from the innermost; first match wins. */
const AREAS: [string, string][] = [
  ["dialog", "[role='dialog']"],
  ["toolbar", "[role='toolbar']"],
  ["search", "input[type='search']"],
  ["tree", "[role='tree']"],
  ["filters", "aside [role='group']"],
  ["actions", "header [role='group']"],
  ["footer", "footer"],
];

/** The focus colors fade in: without transitions a stop reads them at their final value. */
export async function withoutTransitions(page: Page): Promise<void> {
  await page.addStyleTag({ content: "*, *::before { transition: none !important; }" });
}

/** The stop of the focused element; none while the focus is on the body. */
export async function focusedStop(page: Page): Promise<FocusStop | undefined> {
  return page.evaluate((areas): FocusStop | undefined => {
    const element = document.activeElement;
    if (!(element instanceof HTMLElement) || element === document.body) return undefined;
    element.dataset["stop"] ??= String(document.querySelectorAll("[data-stop]").length);
    // The ring of the token, the accent border of the fields or the accent
    // fill of the highlighted entries, on the element or on the box around it.
    const accents = ["127, 194, 160", "43, 116, 84"];
    let visible = false;
    for (let node: HTMLElement | null = element, depth = 0; node && depth < 4; depth++) {
      const style = getComputedStyle(node);
      const ring = style.outlineStyle !== "none" && Number.parseFloat(style.outlineWidth) > 0;
      const tinted = [style.borderTopColor, style.boxShadow, style.backgroundColor].some((value) =>
        accents.some((accent) => value.includes(accent)),
      );
      visible ||= ring || tinted;
      node = node.parentElement;
    }
    const area = areas.find(([, selector]) => element.closest(selector))?.[0] ?? "other";
    const name = (element.getAttribute("aria-label") ?? element.textContent ?? "").trim();
    return { id: element.dataset["stop"], name, area, visible };
  }, AREAS);
}

/**
 * The stops met pressing `key` from the focused element until one comes
 * back, at most `limit`.
 */
export async function sweep(page: Page, key = TAB, limit = 80): Promise<FocusStop[]> {
  const stops: FocusStop[] = [];
  for (let step = 0; step < limit; step++) {
    await page.keyboard.press(key);
    const stop = await focusedStop(page);
    if (!stop) continue;
    if (stops.some((seen) => seen.id === stop.id)) break;
    stops.push(stop);
  }
  return stops;
}

/**
 * Presses Tab until the focus is in `area`, as a user gets there: a focus
 * moved by a script shows no ring (`:focus-visible` is for the keyboard).
 */
export async function tabTo(page: Page, area: string, limit = 80): Promise<FocusStop> {
  for (let step = 0; step < limit; step++) {
    await page.keyboard.press(TAB);
    const stop = await focusedStop(page);
    if (stop?.area === area) return stop;
  }
  throw new Error(`nessuna fermata di Tab in ${area}`);
}
