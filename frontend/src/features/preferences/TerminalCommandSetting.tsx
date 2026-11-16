import { useRef, useState } from "react";

import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import styles from "./Preferences.module.css";
import type { ConfigSettings } from "./useConfigSettings";

/**
 * The command of a custom terminal, with `{cmd}` where the command of
 * Portolano goes (SPEC §7.2): saved when the field is left or with Enter.
 */
export function TerminalCommandSetting({ settings }: { settings: ConfigSettings }) {
  const t = texts.preferences.terminal;
  const saved = settings.value("terminal.command") ?? "";
  const [draft, setDraft] = useState(saved);
  // The value sent last: Enter and then leaving the field send it once.
  const sent = useRef(saved);
  const error = settings.error("terminal.command");
  const commit = () => {
    if (draft === sent.current) return;
    sent.current = draft;
    settings.set("terminal.command", draft);
  };
  return (
    <div className={styles.control}>
      <TextField
        label={t.command}
        mono
        value={draft}
        message={error ? { tone: "error", text: error } : undefined}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") commit();
        }}
      />
      <p className={styles.hint}>{t.commandHint}</p>
    </div>
  );
}
