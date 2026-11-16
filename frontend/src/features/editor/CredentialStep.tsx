import { useState } from "react";

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
import { PASSPHRASE, PASSWORD, PRIVATE_KEY, type SlotKey } from "./secretSlots";

/** Step 2 · Accesso of a credential (inventario §5.5). */
export function CredentialStep() {
  const t = texts.editor.credential;
  const { values, set, errorOf, secrets, askSecret, clearSecret, backend } = useEditorFields();
  const chooseKeyFile = useChooseKeyFile();
  const [browseError, setBrowseError] = useState<string>();

  const row = (key: SlotKey, label: string) => {
    const error = errorOf(
      key === PRIVATE_KEY ? "privateKey" : key === PASSWORD ? "password" : "passphrase",
    );
    return (
      <div className={styles.group}>
        <SecretRow
          label={label}
          state={secrets.state(key)}
          backend={backend}
          onSet={() => askSecret(key)}
          onClear={() => clearSecret(key)}
        />
        {error && (
          <span className={styles.error} role="alert">
            {error}
          </span>
        )}
      </div>
    );
  };

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
      {row(PASSWORD, t.password)}
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
        {values.keyMode === "vault" && row(PRIVATE_KEY, t.key)}
      </Card>
      {values.keyMode !== "none" && row(PASSPHRASE, t.passphrase)}
    </section>
  );
}
