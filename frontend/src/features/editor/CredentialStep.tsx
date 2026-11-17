import { useId, useState } from "react";

import { Button } from "../../components/Button";
import { Card } from "../../components/Card";
import { SecretRow } from "../../components/SecretRow";
import { SegmentedControl } from "../../components/SegmentedControl";
import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useChooseKeyFile } from "../../ipc/mutations";
import { errorMessage, useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import { GroupError } from "./GroupError";
import { PASSPHRASE, PASSWORD, PRIVATE_KEY, type SlotKey } from "./secretSlots";

interface SecretSettingProps {
  slot: SlotKey;
  label: string;
}

/** A secret of the credential, with the error of the backend on it. */
function SecretSetting({ slot, label }: SecretSettingProps) {
  const { errorOf, secrets, askSecret, clearSecret, backend } = useEditorFields();
  const errorId = useId();
  const error = errorOf(
    slot === PRIVATE_KEY ? "privateKey" : slot === PASSWORD ? "password" : "passphrase",
  );
  return (
    <div className={styles.group}>
      <SecretRow
        label={label}
        state={secrets.state(slot)}
        backend={backend}
        onSet={() => askSecret(slot)}
        onClear={() => clearSecret(slot)}
        appliesOnSave
        describedBy={error ? errorId : undefined}
      />
      <GroupError id={errorId} text={error} />
    </div>
  );
}

/** Step 2 · Accesso of a credential (inventario §5.5). */
export function CredentialStep() {
  const t = texts.editor.credential;
  const { values, set, errorOf, backend } = useEditorFields();
  const chooseKeyFile = useChooseKeyFile();
  const [browseError, setBrowseError] = useState<string>();

  const browse = () => {
    setBrowseError(undefined);
    chooseKeyFile
      .mutateAsync()
      .then((path) => {
        if (path !== null) set("keyFile", path);
      })
      .catch((failure: unknown) => setBrowseError(presentFailure(failure).message));
  };

  return (
    <section className={styles.step} aria-label={texts.editor.steps.access}>
      <div className={styles.pair}>
        <TextField
          label={t.user}
          mono
          value={values.username}
          message={errorMessage(errorOf("username"))}
          onChange={(event) => set("username", event.target.value)}
        />
        <TextField
          label={t.domain}
          mono
          placeholder={t.domainPlaceholder}
          value={values.domain}
          message={errorMessage(errorOf("domain"))}
          onChange={(event) => set("domain", event.target.value)}
        />
      </div>
      <SecretSetting slot={PASSWORD} label={t.password} />
      <Card className={styles.keyBox}>
        <span className={styles.fieldLabel}>{t.key}</span>
        <SegmentedControl
          label={t.key}
          value={values.keyMode}
          onChange={(keyMode) => set("keyMode", keyMode)}
          options={[
            { value: "none", label: t.keyModes.none },
            { value: "file", label: t.keyModes.file },
            { value: "vault", label: t.keyModes.vault[backend] },
          ]}
        />
        {values.keyMode === "file" && (
          <div className={styles.addressRow}>
            <TextField
              label={t.keyFile}
              mono
              value={values.keyFile}
              className={styles.grow}
              message={errorMessage(browseError ?? errorOf("keyFile"))}
              onChange={(event) => set("keyFile", event.target.value)}
            />
            <Button loading={chooseKeyFile.isPending} onClick={browse}>
              {t.browse}
            </Button>
          </div>
        )}
        {values.keyMode === "vault" && <SecretSetting slot={PRIVATE_KEY} label={t.key} />}
      </Card>
      {values.keyMode !== "none" && <SecretSetting slot={PASSPHRASE} label={t.passphrase} />}
    </section>
  );
}
