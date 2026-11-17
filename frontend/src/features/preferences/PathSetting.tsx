import { Button } from "../../components/Button";
import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import styles from "./Preferences.module.css";
import type { ConfigKey, ConfigSettings } from "./useConfigSettings";

interface PathSettingProps {
  settings: ConfigSettings;
  configKey: ConfigKey;
  label: string;
  /** What applies while the key is not set, shown as the placeholder. */
  fallback: string;
  hint?: string | undefined;
}

/**
 * A file of the configuration: shown as text, chosen only with the native
 * dialog of Rust (SPEC-UI §1.5, §4.6), and put back to its default.
 */
export function PathSetting({ settings, configKey, label, fallback, hint }: PathSettingProps) {
  const t = texts.preferences;
  const value = settings.value(configKey);
  const error = settings.error(configKey);
  return (
    <div className={styles.control}>
      <div className={styles.pathRow}>
        <TextField
          label={label}
          mono
          readOnly
          value={value ?? ""}
          placeholder={fallback}
          message={error ? { tone: "error", text: error } : undefined}
          className={styles.grow}
        />
        <Button onClick={() => settings.choose(configKey)}>{t.browse}</Button>
        {value !== undefined && (
          <Button variant="ghost" onClick={() => settings.reset(configKey)}>
            {t.reset}
          </Button>
        )}
      </div>
      {hint && <p className={styles.hint}>{hint}</p>}
    </div>
  );
}
