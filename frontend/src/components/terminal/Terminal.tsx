import "@xterm/xterm/css/xterm.css";

import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import { useEffect, useEffectEvent, useId, useRef, useState } from "react";

import { terminalOptions } from "../../design/terminalTheme";
import { texts } from "../../i18n/it";
import { leaveTerminal } from "./leaveTerminal";
import { LinkHint } from "./LinkHint";
import { LinkHintBoundary } from "./LinkHintBoundary";
import styles from "./Terminal.module.css";
import type { TerminalIo } from "./terminalIo";
import { CLIPBOARD_OSC, openableLink, terminalTitle } from "./terminalSafety";

interface TerminalProps {
  io: TerminalIo;
  /** The title the host sets (OSC 0/2), already made visible text. */
  onTitle?: (title: string) => void;
  /** An `http(s)` link the user activated; nothing else reaches it. */
  onOpenLink: (url: string) => void;
  /** A key in the terminal: the background calms down while the user works. */
  onActivity?: () => void;
  label?: string;
}

/** The link under the pointer, where the pointer is. */
interface HoveredLink {
  url: string;
  x: number;
  y: number;
}

/**
 * The integrated terminal (ambiguity 24): xterm.js 6 with its DOM renderer.
 * The title is only text, links open only if `http(s)` through the
 * callback, and OSC 52 cannot touch the clipboard (SPEC-UI §4.8).
 *
 * As in Terminal of macOS, a link opens only with ⌘-click, and the address
 * it would open shows under the pointer (DEC-046.3): the text the host
 * prints, or the label of an OSC 8 link, can say something else.
 */
export function Terminal({
  io,
  onTitle,
  onOpenLink,
  onActivity,
  label = texts.components.terminal.label,
}: TerminalProps) {
  const host = useRef<HTMLDivElement>(null);
  const hintId = useId();
  const title = useEffectEvent((text: string) => onTitle?.(terminalTitle(text)));
  const [hint, setHint] = useState<HoveredLink>();
  const link = useEffectEvent((event: MouseEvent, uri: string) => {
    const url = openableLink(uri);
    if (url && event.metaKey) onOpenLink(url);
  });
  const hover = useEffectEvent((event: MouseEvent, uri: string) => {
    const url = openableLink(uri);
    setHint(url ? { url, x: event.clientX, y: event.clientY } : undefined);
  });
  const leave = useEffectEvent(() => setHint(undefined));
  const activity = useEffectEvent(() => onActivity?.());

  useEffect(() => {
    const element = host.current;
    if (!element) return undefined;
    const terminal = new XTerm({
      ...terminalOptions,
      linkHandler: {
        activate: (event, uri) => link(event, uri),
        hover: (event, uri) => hover(event, uri),
        leave: () => leave(),
        allowNonHttpProtocols: false,
      },
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.loadAddon(
      new WebLinksAddon((event, uri) => link(event, uri), {
        hover: (event, uri) => hover(event, uri),
        leave: () => leave(),
      }),
    );
    const subscriptions = [
      terminal.parser.registerOscHandler(CLIPBOARD_OSC, () => true),
      terminal.onTitleChange((text) => title(text)),
      terminal.onData((data) => {
        io.input(data);
        activity();
      }),
      terminal.onResize(({ cols, rows }) => io.resize(cols, rows)),
    ];
    terminal.attachCustomKeyEventHandler((event) => {
      if (event.type !== "keydown" || event.key !== "Tab" || !event.ctrlKey) return true;
      event.preventDefault();
      leaveTerminal(element, event.shiftKey ? -1 : 1);
      return false;
    });
    terminal.open(element);
    // xterm.js names its input "Terminal input", in English.
    terminal.textarea?.setAttribute("aria-label", texts.components.terminal.input);
    terminal.textarea?.setAttribute("aria-describedby", hintId);
    const unsubscribe = io.subscribe((data) => terminal.write(data));
    const resize = new ResizeObserver(() => fit.fit());
    resize.observe(element);
    // The cells are measured with the font: fit again once it has loaded.
    void document.fonts.ready.then(() => fit.fit());
    return () => {
      resize.disconnect();
      unsubscribe();
      subscriptions.forEach((subscription) => subscription.dispose());
      terminal.dispose();
      // The links of the terminal gone, so is the hint of one of them.
      setHint(undefined);
    };
  }, [io, hintId]);

  return (
    <div role="group" aria-label={label} className={styles.frame}>
      <div ref={host} className={styles.terminal} />
      <span id={hintId} className={styles.hint}>
        {texts.components.terminal.leave}
      </span>
      {hint && (
        // A new boundary for each link: a hint that failed does not hide the next.
        <LinkHintBoundary key={hint.url}>
          <LinkHint url={hint.url} x={hint.x} y={hint.y} />
        </LinkHintBoundary>
      )}
    </div>
  );
}
