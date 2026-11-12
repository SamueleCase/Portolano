import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { Button } from "./Button";
import { classNames } from "./classNames";
import styles from "./KeyValueEditor.module.css";
import { SecretBadge, type SecretState, type VaultBackend } from "./SecretBadge";
import { visibleText } from "./visibleText";

/** A custom property or a free ssh option. */
export interface KeyValueEntry {
  id: string;
  key: string;
  /** Only custom properties: a secret shows its state, never its value. */
  type: "text" | "secret";
  value: string;
  secret?: SecretState;
  /** The message of the backend for this entry, such as a refused option. */
  error?: string | undefined;
}

interface KeyValueEditorProps {
  /** The accessible name of the table, such as "Proprietà custom". */
  label: string;
  /** `properties`: name, type, value; `options`: key = value in mono (inventario §4.26). */
  variant: "properties" | "options";
  entries: readonly KeyValueEntry[];
  backend: VaultBackend;
  addLabel: string;
  onChange: (id: string, change: { key?: string; value?: string }) => void;
  onReplaceSecret: (id: string) => void;
  onRemove: (id: string) => void;
  onAdd: () => void;
}

/** The editor of the custom properties and of the free ssh options. */
export function KeyValueEditor({
  label,
  variant,
  entries,
  backend,
  addLabel,
  onChange,
  onReplaceSecret,
  onRemove,
  onAdd,
}: KeyValueEditorProps) {
  const t = texts.components.keyValue;
  const mono = variant === "options";
  return (
    <div className={styles.editor}>
      <div role="table" aria-label={label} className={classNames(styles.table, styles[variant])}>
        {entries.map((entry) => {
          const name = visibleText(entry.key);
          return (
            <div key={entry.id} role="row" className={styles.row}>
              <span role="cell" className={styles.cell}>
                <input
                  aria-label={t.name}
                  className={classNames(styles.input, mono && styles.mono)}
                  value={entry.key}
                  aria-invalid={entry.error ? true : undefined}
                  onChange={(event) => onChange(entry.id, { key: event.target.value })}
                />
              </span>
              {variant === "properties" && (
                <span role="cell" className={classNames(styles.cell, styles.type)}>
                  {t.types[entry.type]}
                </span>
              )}
              <span role="cell" className={styles.cell}>
                {entry.type === "secret" ? (
                  <span className={styles.secret}>
                    <SecretBadge
                      state={entry.secret ?? "absent"}
                      backend={backend}
                      variant="compact"
                    />
                    <Button variant="ghost" onClick={() => onReplaceSecret(entry.id)}>
                      {t.replace}
                    </Button>
                  </span>
                ) : (
                  <input
                    aria-label={variant === "properties" ? t.value : `${name} ${t.value}`}
                    className={classNames(styles.input, mono && styles.mono)}
                    value={entry.value}
                    onChange={(event) => onChange(entry.id, { value: event.target.value })}
                  />
                )}
              </span>
              <span role="cell">
                <button
                  type="button"
                  className={styles.remove}
                  aria-label={interpolate(t.remove, { name })}
                  onClick={() => onRemove(entry.id)}
                >
                  ×
                </button>
              </span>
              {entry.error && (
                <span role="cell" className={styles.error}>
                  {entry.error}
                </span>
              )}
            </div>
          );
        })}
      </div>
      <Button variant="add" className={styles.add} onClick={onAdd}>
        {addLabel}
      </Button>
    </div>
  );
}
