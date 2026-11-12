import { visibleText } from "../visibleText";

/**
 * A link printed by the remote host, only if it is `http(s)`: anything else
 * (`file:`, `javascript:`, custom schemes) is never opened (SPEC-UI §4.8).
 */
export function openableLink(uri: string): string | undefined {
  try {
    const url = new URL(uri);
    return url.protocol === "http:" || url.protocol === "https:" ? url.href : undefined;
  } catch {
    return undefined;
  }
}

/** The window title set by the remote host (OSC 0/2), as plain visible text. */
export function terminalTitle(title: string): string {
  return visibleText(title);
}

/** OSC 52, the escape that would read or write the clipboard: disabled. */
export const CLIPBOARD_OSC = 52;
