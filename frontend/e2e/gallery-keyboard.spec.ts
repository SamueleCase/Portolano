import { expect, test } from "./fixtures";
import { openGallery } from "./galleryPage";

/** Elements a user must be able to reach with Tab (one stop per radio group). */
const CONTROLS = [
  "a[href]",
  "button:not([tabindex='-1'])",
  "input:not([type='hidden'])",
  "textarea:not(.xterm-helper-textarea)",
  "[role='combobox']:not(input)",
  "[tabindex='0']",
].join(",");

/**
 * WebKit, like Safari, moves with Tab only among text fields unless the user
 * turns on keyboard navigation; Option+Tab moves among every control.
 */
const TAB = "Alt+Tab";

/** A stop of the Tab sweep. */
interface Stop {
  id: string;
  /** The terminal input, left with Ctrl+Tab since Tab belongs to the shell. */
  terminal: boolean;
  visible: boolean;
}

test("every control of the gallery is reached with Tab and shows its focus", async ({ page }) => {
  test.slow();
  await openGallery(page);
  // The focus colors fade in: read them at once, at their final value.
  await page.addStyleTag({ content: "*, *::before { transition: none !important; }" });
  const total = await page.evaluate((selector) => {
    const controls = [...document.querySelectorAll<HTMLElement>(selector)].filter(
      (element) => !element.matches(":disabled") && element.getClientRects().length > 0,
    );
    controls.forEach((element, index) => {
      element.dataset["control"] = String(index);
    });
    return controls.length;
  }, CONTROLS);

  const stops: Stop[] = [];
  let key = TAB;
  for (let step = 0; step < total + 20; step++) {
    await page.keyboard.press(key);
    const stop = await page.evaluate((): Stop | undefined => {
      const element = document.activeElement;
      if (!(element instanceof HTMLElement) || element === document.body) return undefined;
      // The ring of the token, the accent border of the fields or the accent
      // fill of the highlighted entries, on the element or on the box around
      // it (tag input, terminal).
      const accents = ["127, 194, 160", "43, 116, 84"];
      let visible = false;
      for (let node: HTMLElement | null = element, depth = 0; node && depth < 4; depth++) {
        const style = getComputedStyle(node);
        const ring = style.outlineStyle !== "none" && Number.parseFloat(style.outlineWidth) > 0;
        const tinted = [style.borderTopColor, style.boxShadow, style.backgroundColor].some(
          (value) => accents.some((accent) => value.includes(accent)),
        );
        visible ||= ring || tinted;
        node = node.parentElement;
      }
      const terminal = element.classList.contains("xterm-helper-textarea");
      return { id: element.dataset["control"] ?? element.tagName, terminal, visible };
    });
    if (!stop) continue;
    if (stops.some((seen) => seen.id === stop.id)) break;
    stops.push(stop);
    key = stop.terminal ? "Control+Tab" : TAB;
  }

  const reached = new Set(stops.map((stop) => stop.id));
  const names = (ids: string[]) =>
    page.evaluate(
      (wanted) =>
        wanted.map((id) => {
          const element = document.querySelector<HTMLElement>(`[data-control="${id}"]`);
          return element?.getAttribute("aria-label") ?? element?.textContent ?? id;
        }),
      ids,
    );
  const missed = Array.from({ length: total }, (_, index) => String(index)).filter(
    (id) => !reached.has(id),
  );
  expect(await names(missed), "controlli non raggiungibili con Tab").toEqual([]);
  const unseen = stops.filter((stop) => !stop.visible).map((stop) => stop.id);
  expect(await names(unseen), "controlli senza fuoco visibile").toEqual([]);
  expect(
    stops.some((stop) => stop.terminal),
    "il terminale si raggiunge e si lascia",
  ).toBe(true);
});
