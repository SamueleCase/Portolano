/**
 * The file names the `portolano-icon://` scheme serves (SPEC-UI §4.7): the
 * SHA-256 of the image and its extension. Anything else from the document
 * is not turned into a URL.
 */
const CUSTOM_ICON_FILE = /^[0-9a-f]{64}\.(png|jpg|gif|tiff|svg|pdf)$/;

/** The URL of a custom icon, or `undefined` if `fileName` is not a valid icon name. */
export function customIconUrl(fileName: string): string | undefined {
  return CUSTOM_ICON_FILE.test(fileName) ? `portolano-icon://localhost/${fileName}` : undefined;
}
