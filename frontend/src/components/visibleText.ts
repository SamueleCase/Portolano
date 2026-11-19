/**
 * Strings of the document come from untrusted sources (DEC-024): a name, a
 * host or a note could carry control characters, separators of lines,
 * zero-width characters, or bidi controls that show the text in another
 * order than it has. They read here exactly as the CLI prints them
 * (`portolano_core::text::visible`, DEC-026.4, DEC-047.2).
 *
 * The rule: every component that receives a text which may come from the
 * document (a name, a path, a host, a label of a property, a title, a
 * tooltip) makes it visible itself, so the screens never have to remember.
 * The function is idempotent, so a text already made visible is unchanged.
 */

/**
 * C0 controls, DEL and C1 controls: Rust's `char::is_control` (category
 * Cc). Exported for the checks that must refuse what the backend refuses.
 */
export function isControl(codePoint: number): boolean {
  return codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f);
}

/**
 * The invisible characters that are not controls: the line and paragraph
 * separators (Zl, Zp) and the format characters (Cf), bidi and zero-width
 * ones included, as ranges of code points. The same list as
 * `portolano_core::text` (DEC-047.2); a test checks it against the Unicode
 * properties of the engine.
 */
const INVISIBLE_FORMAT: readonly (readonly [number, number])[] = [
  [0x00ad, 0x00ad],
  [0x0600, 0x0605],
  [0x061c, 0x061c],
  [0x06dd, 0x06dd],
  [0x070f, 0x070f],
  [0x0890, 0x0891],
  [0x08e2, 0x08e2],
  [0x180e, 0x180e],
  [0x200b, 0x200f],
  [0x2028, 0x202e],
  [0x2060, 0x2064],
  [0x2066, 0x206f],
  [0xfeff, 0xfeff],
  [0xfff9, 0xfffb],
  [0x110bd, 0x110bd],
  [0x110cd, 0x110cd],
  [0x13430, 0x1343f],
  [0x1bca0, 0x1bca3],
  [0x1d173, 0x1d17a],
  [0xe0001, 0xe0001],
  [0xe0020, 0xe007f],
];

/** Whether `codePoint` is a separator of lines or paragraphs or a format character. */
function isInvisibleFormat(codePoint: number): boolean {
  return INVISIBLE_FORMAT.some(([first, last]) => codePoint >= first && codePoint <= last);
}

/**
 * The zero-width non-joiner and joiner (U+200C, U+200D): format characters
 * that compose emoji and join or keep apart the letters of some scripts.
 * The text of a document keeps them (DEC-047.5).
 */
export const JOINERS: ReadonlySet<number> = new Set([0x200c, 0x200d]);

/**
 * `text` with each control, separator and format character written as
 * `\u{1b}`, except the joiners (`JOINERS`).
 */
export function visibleText(text: string): string {
  let shown = "";
  for (const character of text) {
    const codePoint = character.codePointAt(0) ?? 0;
    shown +=
      isControl(codePoint) || (isInvisibleFormat(codePoint) && !JOINERS.has(codePoint))
        ? `\\u{${codePoint.toString(16)}}`
        : character;
  }
  return shown;
}
