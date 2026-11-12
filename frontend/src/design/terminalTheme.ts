import type { ITerminalOptions } from "@xterm/xterm";

import { tokens } from "./tokens";

/**
 * The look of the terminal from the tokens (inventario §4.26 `TerminalView`,
 * ambiguity 24): opaque, IBM Plex Mono 13/1.65, text in the secondary color,
 * cursor and selection in the accent.
 */
export const terminalOptions: ITerminalOptions = {
  fontFamily: tokens["--font-mono"],
  fontSize: Number.parseFloat(tokens["--font-size-body"]),
  lineHeight: Number(tokens["--line-height-terminal"]),
  cursorBlink: true,
  allowProposedApi: false,
  theme: {
    background: tokens["--color-terminal"],
    foreground: tokens["--color-text-secondary"],
    cursor: tokens["--color-accent-text"],
    cursorAccent: tokens["--color-terminal"],
    selectionBackground: tokens["--color-active"],
    green: tokens["--color-node-ssh"],
    brightGreen: tokens["--color-ok"],
    yellow: tokens["--color-warn"],
    red: tokens["--color-error"],
    brightRed: tokens["--color-error-hover"],
    blue: tokens["--color-node-rdp"],
    cyan: tokens["--color-info"],
  },
};
