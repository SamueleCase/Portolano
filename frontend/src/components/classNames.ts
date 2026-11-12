/** The class names that apply, in order: falsy entries are dropped. */
export function classNames(...names: ReadonlyArray<string | false | null | undefined>): string {
  return names.filter(Boolean).join(" ");
}
