import { afterAll, beforeAll } from "vitest";

/**
 * For the tests that open the `NodePicker`: it scrolls its active option
 * into view, and jsdom, which does not lay out, has no `scrollIntoView`.
 */
export function stubScrollIntoView(): void {
  beforeAll(() => {
    Element.prototype.scrollIntoView = () => undefined;
  });
  afterAll(() => {
    Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  });
}
