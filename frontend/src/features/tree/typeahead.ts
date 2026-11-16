/** How long the letters typed in the tree make one word (WAI-ARIA tree pattern). */
export const TYPEAHEAD_RESET_MS = 500;

/**
 * The first of `names` after position `from` (going round) that starts with
 * `prefix`, ignoring case; with a repeated single letter the search moves on
 * from the current item, as in the file managers.
 */
export function typeaheadMatch(
  names: readonly string[],
  from: number,
  prefix: string,
): number | undefined {
  const wanted = prefix.toLocaleLowerCase("it-IT");
  if (wanted === "" || names.length === 0) return undefined;
  const repeated = [...wanted].every((letter) => letter === wanted[0]);
  const start = repeated || wanted.length === 1 ? from + 1 : from;
  for (let step = 0; step < names.length; step += 1) {
    const index = (start + step) % names.length;
    const name = names[index]?.toLocaleLowerCase("it-IT") ?? "";
    if (name.startsWith(repeated ? (wanted[0] ?? "") : wanted)) return index;
  }
  return undefined;
}
