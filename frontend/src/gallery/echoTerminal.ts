import type { TerminalIo } from "../components/terminal/terminalIo";

const ESC = "\u001b";
const BEL = "\u0007";

/** What the gallery terminal prints first: colors, a title, a link and OSC 52. */
function greeting(title: string, link: string): string {
  return [
    `${ESC}]0;${title}${BEL}`,
    `${ESC}[32mdeploy@web01${ESC}[0m:${ESC}[34m~${ESC}[0m$ uptime\r\n`,
    " 10:42:07 up 12 days,  3:14,  1 user,  load average: 0.08, 0.03, 0.01\r\n",
    `Runbook: ${link}\r\n`,
    // A host trying to write the clipboard: the terminal ignores it.
    `${ESC}]52;c;aGVsbG8=${BEL}`,
    `${ESC}[32mdeploy@web01${ESC}[0m:${ESC}[34m~${ESC}[0m$ `,
  ].join("");
}

/**
 * A session of the gallery: prints a greeting and echoes what is typed,
 * without any process behind it.
 */
export function echoTerminal(title: string, link: string): TerminalIo {
  const encoder = new TextEncoder();
  let output: ((data: Uint8Array) => void) | undefined;
  return {
    subscribe(onOutput) {
      output = onOutput;
      onOutput(encoder.encode(greeting(title, link)));
      return () => {
        output = undefined;
      };
    },
    input(data) {
      output?.(encoder.encode(data === "\r" ? "\r\n$ " : data));
    },
    resize() {},
  };
}
