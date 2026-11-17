import { Button } from "../../components/Button";
import { type KeyValueEntry, KeyValueEditor } from "../../components/KeyValueEditor";
import { texts } from "../../i18n/it";
import { useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import type { PropertyRow } from "./editorValues";
import { propertySlot } from "./secretSlots";

/**
 * The custom properties (inventario §5.5): text ones are written here,
 * secret ones show their state and take their value from the password
 * dialog, staged until "Salva".
 */
export function PropertiesEditor() {
  const t = texts.editor.general;
  const { values, set, errorOf, secrets, askSecret, clearSecret, backend, newRowId } =
    useEditorFields();
  const rows = values.properties;

  const entries: KeyValueEntry[] = rows.map((row) => ({
    id: row.id,
    key: row.name,
    type: row.secret ? "secret" : "text",
    value: row.text,
    secret: secrets.state(propertySlot(row)),
  }));
  const add = (secret: boolean) => {
    const row: PropertyRow = { id: newRowId(), name: "", secret, text: "", saved: null };
    set("properties", [...rows, row]);
  };
  const error = errorOf("properties");

  return (
    <div className={styles.group}>
      <span className={styles.fieldLabel}>{t.properties}</span>
      <KeyValueEditor
        label={t.properties}
        variant="properties"
        entries={entries}
        backend={backend}
        addLabel={t.addProperty}
        onAdd={() => add(false)}
        onChange={(id, change) =>
          set(
            "properties",
            rows.map((row) =>
              row.id === id
                ? { ...row, name: change.key ?? row.name, text: change.value ?? row.text }
                : row,
            ),
          )
        }
        onReplaceSecret={(id) => {
          const row = rows.find((candidate) => candidate.id === id);
          if (row) askSecret(propertySlot(row));
        }}
        onRemove={(id) => {
          const row = rows.find((candidate) => candidate.id === id);
          // A removed secret property drops its staged value; a saved one is
          // removed from the document on "Salva".
          if (row?.secret) clearSecret(propertySlot(row));
          set(
            "properties",
            rows.filter((candidate) => candidate.id !== id),
          );
        }}
      />
      <Button variant="add" className={styles.addSecret} onClick={() => add(true)}>
        {t.addSecretProperty}
      </Button>
      {error && (
        <span className={styles.error} role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
