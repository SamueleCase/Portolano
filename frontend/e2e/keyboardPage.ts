import type { Page } from "@playwright/test";

/**
 * The helpers of the keyboard e2e, for the gallery and the main window.
 *
 * WebKit, like Safari, moves with Tab only among text fields unless the user
 * turns on keyboard navigation; Option+Tab moves among every control
 * (DEC-032.4). The app itself should reach every control with Tab
 * (DEC-036.1), still to be verified live.
 */
export const TAB = "Alt+Tab";
export const BACK_TAB = "Alt+Shift+Tab";

/** The elements a user must be able to reach with Tab (one stop per radio group, toolbar, tree). */
const CONTROLS = [
  "a[href]",
  "button:not([tabindex='-1'])",
  "input:not([type='hidden'])",
  "textarea:not(.xterm-helper-textarea)",
  "[role='combobox']:not(input)",
  "[tabindex='0']",
].join(",");

/** The focused element, as a stop of a sweep with Tab. */
export interface FocusStop {
  /** The same for the same element across the sweep: its mark, see {@link markControls}. */
  id: string;
  /** The accessible name, or the text, of the element. */
  name: string;
  /** The landmark around it: the toolbar, the tree, a dialog… (`AREAS`). */
  area: string;
  /** Whether focusing it changes how it, or the box around it, looks. */
  visible: boolean;
  /** The input of a terminal, which keeps Tab for the shell: left with Ctrl+Tab. */
  terminal: boolean;
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

/**
 * Marks every control of the page that is shown and enabled with
 * `data-control`, its position; returns how many there are.
 */
export async function markControls(page: Page): Promise<number> {
  return page.evaluate((selector) => {
    const controls = [...document.querySelectorAll<HTMLElement>(selector)].filter(
      (element) => !element.matches(":disabled") && element.getClientRects().length > 0,
    );
    controls.forEach((element, index) => {
      element.dataset["control"] = String(index);
    });
    return controls.length;
  }, CONTROLS);
}

/** The elements that can take the focus, whose look is written down before a key. */
const FOCUSABLE = "a[href], button, input, textarea, select, [tabindex], [contenteditable='true']";

declare global {
  interface Window {
    /** How an element and the three boxes around it look: outline, shadow, borders, background. */
    __portolanoLookOf?: (element: HTMLElement) => string;
    /** How each focusable element looked before the last key, without the focus. */
    __portolanoUnfocused?: WeakMap<HTMLElement, string>;
  }
}

/** Puts in the page the measure of how an element looks, once. */
function installLookOf(): void {
  window.__portolanoLookOf ??= (element) => {
    const seen: string[] = [];
    for (let node: HTMLElement | null = element, depth = 0; node && depth < 4; depth++) {
      const style = getComputedStyle(node);
      seen.push(
        [
          style.outlineStyle,
          style.outlineWidth,
          style.outlineColor,
          style.boxShadow,
          style.borderTopColor,
          style.borderBottomColor,
          style.backgroundColor,
        ].join("|"),
      );
      node = node.parentElement;
    }
    return seen.join("/");
  };
}

/**
 * Writes down how every focusable element looks now, without the focus:
 * the stop the next key reaches is then judged against its own look before
 * the key, never by moving the focus (DEC-047.4).
 */
async function rememberUnfocused(page: Page): Promise<void> {
  await page.evaluate(installLookOf);
  await page.evaluate((selector) => {
    const lookOf = window.__portolanoLookOf;
    if (!lookOf) return;
    const looks = new WeakMap<HTMLElement, string>();
    for (const element of document.querySelectorAll<HTMLElement>(selector)) {
      if (!element.contains(document.activeElement)) looks.set(element, lookOf(element));
    }
    window.__portolanoUnfocused = looks;
  }, FOCUSABLE);
}

/**
 * The stop of the focused element; none while the focus is on the body.
 *
 * Its focus is visible if the element, or one of the three boxes around it,
 * looks different now than before the key that focused it: the outline, the
 * shadow, a border or the background change. An element that did not exist
 * before the key (a dialog it opened) has no look of its own to compare: it
 * loses the focus for a moment to be seen without it, then gets it back,
 * which a ring shown only on `:focus-visible` may not survive in WebKit.
 */
async function focusedStop(page: Page): Promise<FocusStop | undefined> {
  await page.evaluate(installLookOf);
  return page.evaluate((areas): FocusStop | undefined => {
    const element = document.activeElement;
    const lookOf = window.__portolanoLookOf;
    if (!(element instanceof HTMLElement) || element === document.body || !lookOf) {
      return undefined;
    }
    const focused = lookOf(element);
    let unfocused = window.__portolanoUnfocused?.get(element);
    if (unfocused === undefined) {
      element.blur();
      unfocused = lookOf(element);
      element.focus({ preventScroll: true });
    }
    element.dataset["control"] ??= `other-${document.querySelectorAll("[data-control]").length}`;
    const area = areas.find(([, selector]) => element.closest(selector))?.[0] ?? "other";
    const name = (element.getAttribute("aria-label") ?? element.textContent ?? "").trim();
    return {
      id: element.dataset["control"],
      name,
      area,
      visible: focused !== unfocused,
      terminal: element.classList.contains("xterm-helper-textarea"),
    };
  }, AREAS);
}

/** Presses `key` and returns the stop it reaches, its focus judged against its look before. */
export async function press(page: Page, key: string): Promise<FocusStop | undefined> {
  await rememberUnfocused(page);
  await page.keyboard.press(key);
  return focusedStop(page);
}

/**
 * The stops met pressing `key` from the focused element until one comes
 * back, at most `limit`. A terminal is left with Ctrl+Tab, as a user does.
 */
export async function sweep(page: Page, key = TAB, limit = 80): Promise<FocusStop[]> {
  const stops: FocusStop[] = [];
  let next = key;
  for (let step = 0; step < limit; step++) {
    const stop = await press(page, next);
    if (!stop) continue;
    if (stops.some((seen) => seen.id === stop.id)) break;
    stops.push(stop);
    next = stop.terminal ? (key === BACK_TAB ? "Control+Shift+Tab" : "Control+Tab") : key;
  }
  return stops;
}

/**
 * The names of the controls marked by {@link markControls} (`total` of
 * them) that none of `stops` reached.
 */
export async function missedControls(
  page: Page,
  total: number,
  stops: FocusStop[],
): Promise<string[]> {
  const reached = new Set(stops.map((stop) => stop.id));
  const missed = Array.from({ length: total }, (_, index) => String(index)).filter(
    (id) => !reached.has(id),
  );
  return page.evaluate(
    (ids) =>
      ids.map((id) => {
        const element = document.querySelector<HTMLElement>(`[data-control="${id}"]`);
        return element?.getAttribute("aria-label") ?? element?.textContent ?? id;
      }),
    missed,
  );
}

/**
 * Presses Tab until the focus is in `area`, as a user gets there: a focus
 * moved by a script shows no ring (`:focus-visible` is for the keyboard).
 */
export async function tabTo(page: Page, area: string, limit = 80): Promise<FocusStop> {
  for (let step = 0; step < limit; step++) {
    const stop = await press(page, TAB);
    if (stop?.area === area) return stop;
  }
  throw new Error(`nessuna fermata di Tab in ${area}`);
}
