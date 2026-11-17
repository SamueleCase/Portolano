import { focusableIn } from "../../components/focusable";

/**
 * Where the focus goes in the form of the editor (SPEC-UI §1.6): the step
 * shown is the `<section>` right inside the form.
 */

/** The first field of the step shown in `form`, else its first control. */
export function firstFieldOfStep(form: HTMLElement): HTMLElement | undefined {
  const step = form.querySelector<HTMLElement>(":scope > section");
  if (!step) return undefined;
  const controls = focusableIn(step);
  return controls.find((control) => control.matches("input, textarea")) ?? controls[0];
}

/** Whether `element` is marked invalid or described by an error (`role="alert"`). */
function hasError(element: HTMLElement): boolean {
  if (element.getAttribute("aria-invalid") === "true") return true;
  const ids = element.getAttribute("aria-describedby")?.split(/\s+/) ?? [];
  return ids.some((id) => document.getElementById(id)?.getAttribute("role") === "alert");
}

/**
 * The control of the first error in `form`: a field marked invalid, a
 * control its error describes (the option that holds the tab stop, in a
 * group of radios), or the first control of a named group it describes.
 */
export function controlWithError(form: HTMLElement): HTMLElement | undefined {
  const marked = [
    ...form.querySelectorAll<HTMLElement>("[aria-invalid='true'], [aria-describedby]"),
  ].find(hasError);
  if (!marked) return undefined;
  switch (marked.getAttribute("role")) {
    case "group":
      return focusableIn(marked)[0];
    case "radio":
      return (
        marked.closest("[role='radiogroup']")?.querySelector<HTMLElement>("[tabindex='0']") ??
        marked
      );
    default:
      return marked;
  }
}
