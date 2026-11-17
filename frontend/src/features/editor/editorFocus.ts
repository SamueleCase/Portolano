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

/**
 * The control of the first error in `form`: a field marked invalid, or the
 * first control of a group that its error describes.
 */
export function controlWithError(form: HTMLElement): HTMLElement | undefined {
  const marked = form.querySelector<HTMLElement>(
    "[aria-invalid='true'], [role='group'][aria-describedby]",
  );
  if (!marked) return undefined;
  return marked.getAttribute("role") === "group" ? focusableIn(marked)[0] : marked;
}
