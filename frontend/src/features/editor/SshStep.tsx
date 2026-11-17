import { GroupHeader } from "../../components/GroupHeader";
import { type KeyValueEntry, KeyValueEditor } from "../../components/KeyValueEditor";
import { TextField } from "../../components/TextField";
import { ToggleRow } from "../../components/ToggleRow";
import { texts } from "../../i18n/it";
import { ConnectionTarget } from "./ConnectionTarget";
import { CredentialSetting } from "./CredentialSetting";
import { errorMessage, useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import { JumpHostList } from "./JumpHostList";

/** Step 2 · Connessione of an SSH connection (inventario §5.5). */
export function SshStep() {
  const t = texts.editor.ssh;
  const { values, set, errorOf, refusedOption, backend, newRowId } = useEditorFields();
  const optionsError = errorOf("options");
  // The row of the option the backend named, if it is still there.
  const refused = values.options.some((option) => option.key === refusedOption)
    ? refusedOption
    : undefined;
  const entries: KeyValueEntry[] = values.options.map((option) => ({
    id: option.id,
    key: option.key,
    type: "text",
    value: option.value,
    error: refused !== undefined && option.key === refused ? optionsError : undefined,
  }));

  return (
    <section className={styles.step} aria-label={texts.editor.steps.connection}>
      <GroupHeader>{texts.editor.steps.connection}</GroupHeader>
      <ConnectionTarget protocol="ssh" />
      <CredentialSetting
        label={texts.editor.connection.credential}
        value={values.credential}
        onChange={(credential) => set("credential", credential)}
        ownPassword
        error={errorOf("credential")}
      />
      <JumpHostList />
      <GroupHeader>{t.advanced}</GroupHeader>
      <div className={styles.toggles}>
        <ToggleRow
          label={t.agentForwarding}
          checked={values.agentForwarding}
          onChange={(on) => set("agentForwarding", on)}
        />
        <ToggleRow
          label={t.compression}
          checked={values.compression}
          onChange={(on) => set("compression", on)}
        />
        <ToggleRow label={t.x11} checked={values.x11} onChange={(on) => set("x11", on)} />
        <ToggleRow
          label={t.x11Trusted}
          description={t.x11TrustedHint}
          checked={values.x11 && values.x11Trusted}
          disabled={!values.x11}
          onChange={(on) => set("x11Trusted", on)}
        />
      </div>
      {/* "Keepalive ogni [30] secondi (vuoto = disattivato)", as a sentence (inventario §5.5). */}
      <div className={styles.keepalive}>
        <span aria-hidden="true">{t.keepaliveBefore}</span>
        <TextField
          label={t.keepalive}
          hideLabel
          mono
          variant="numeric"
          inputMode="numeric"
          value={values.keepalive}
          message={errorMessage(errorOf("keepalive"))}
          onChange={(event) => set("keepalive", event.target.value)}
        />
        <span aria-hidden="true">{t.keepaliveAfter}</span>
      </div>
      <div className={styles.group}>
        <span className={styles.fieldLabel}>{t.options}</span>
        <KeyValueEditor
          label={t.options}
          variant="options"
          entries={entries}
          backend={backend}
          addLabel={t.addOption}
          onAdd={() => set("options", [...values.options, { id: newRowId(), key: "", value: "" }])}
          onChange={(id, change) =>
            set(
              "options",
              values.options.map((option) =>
                option.id === id
                  ? {
                      ...option,
                      key: change.key ?? option.key,
                      value: change.value ?? option.value,
                    }
                  : option,
              ),
            )
          }
          // The free options are text only: no secret to replace.
          onReplaceSecret={() => undefined}
          onRemove={(id) =>
            set(
              "options",
              values.options.filter((option) => option.id !== id),
            )
          }
        />
        {optionsError && refused === undefined && (
          <span className={styles.error} role="alert">
            {optionsError}
          </span>
        )}
      </div>
    </section>
  );
}
