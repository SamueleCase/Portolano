const HEX_COLOR = /^#[0-9a-f]{6}$/i;

/**
 * The color of a node as CSS, only if it is a plain `#RRGGBB`: the value
 * comes from the document and must never become arbitrary CSS.
 */
export function nodeColor(value: string | null | undefined): string | undefined {
  return value && HEX_COLOR.test(value) ? value : undefined;
}
