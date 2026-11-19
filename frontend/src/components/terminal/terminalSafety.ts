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

/** What the hint of a link shows: the host first, then the address, and the credentials it carries. */
export interface LinkParts {
  /** The host, with its port: never shortened. */
  host: string;
  /** The scheme, the host and the path, the path shortened in the middle. */
  address: string;
  /** The user of `utente@host`, if the address carries credentials. */
  user: string | null;
}

/** The longest path the hint shows; a longer one loses its middle. */
const SHOWN_PATH_CHARS = 48;

/**
 * The parts of an openable link (`openableLink`) for its hint (DEC-047.3):
 * `https://intranet.example.com@evil.example/` goes to `evil.example`, with
 * a user `intranet.example.com` that must show.
 */
export function linkParts(href: string): LinkParts {
  const url = new URL(href);
  const path = shortenMiddle(`${url.pathname}${url.search}${url.hash}`, SHOWN_PATH_CHARS);
  const credentials = url.username !== "" || url.password !== "";
  return {
    host: url.host,
    address: `${url.protocol}//${url.host}${path}`,
    user: credentials ? decodeURIComponent(url.username) : null,
  };
}

/** `text` of at most `max` characters, its middle replaced by "…" if longer. */
export function shortenMiddle(text: string, max: number): string {
  const characters = [...text];
  if (characters.length <= max) return text;
  const tail = Math.floor((max - 1) / 2);
  const head = max - 1 - tail;
  return `${characters.slice(0, head).join("")}…${characters.slice(-tail).join("")}`;
}
