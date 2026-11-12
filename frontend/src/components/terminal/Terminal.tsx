import "@xterm/xterm/css/xterm.css";

import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import { useEffect, useEffectEvent, useId, useRef } from "react";

import { terminalOptions } from "../../design/terminalTheme";
import { texts } from "../../i18n/it";
import { leaveTerminal } from "./leaveTerminal";
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

/**
 * The integrated terminal (ambiguity 24): xterm.js 6 with its DOM renderer.
 * The title is only text, links open only if `http(s)` through the
 * callback, and OSC 52 cannot touch the clipboard (SPEC-UI §4.8).
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
  const link = useEffectEvent((uri: string) => {
    const url = openableLink(uri);
    if (url) onOpenLink(url);
  });
  const activity = useEffectEvent(() => onActivity?.());

  useEffect(() => {
    const element = host.current;
    if (!element) return undefined;
    const terminal = new XTerm({
      ...terminalOptions,
      linkHandler: { activate: (_event, uri) => link(uri), allowNonHttpProtocols: false },
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.loadAddon(new WebLinksAddon((_event, uri) => link(uri)));
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
    };
  }, [io, hintId]);

  return (
    <div role="group" aria-label={label} className={styles.frame}>
      <div ref={host} className={styles.terminal} />
      <span id={hintId} className={styles.hint}>
        {texts.components.terminal.leave}
      </span>
    </div>
  );
}
