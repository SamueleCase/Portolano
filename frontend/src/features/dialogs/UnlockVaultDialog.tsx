import { useQuery } from "@tanstack/react-query";
import { useRef, useState } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { LockIcon } from "../../components/icons/LockIcon";
import { TextField } from "../../components/TextField";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useUnlockVault } from "../../ipc/mutations";
import { vaultStatusQuery } from "../../ipc/queries";
import styles from "./Dialogs.module.css";

interface UnlockVaultDialogProps {
  open: boolean;
  /** "Apri senza segreti", Esc or a click on the veil: the vault stays locked. */
  onClose: () => void;
  /** The vault opened with the master password typed. */
  onUnlocked: () => void;
}

/**
 * The unlock dialog of the file vault (inventario §5.10): the master
 * password goes to `unlock_vault` in one command and its field empties at
 * once; a wrong one shows the error under the field.
 */
export function UnlockVaultDialog(props: UnlockVaultDialogProps) {
  return props.open ? <OpenUnlockVaultDialog {...props} /> : null;
}

function OpenUnlockVaultDialog({ onClose, onUnlocked }: UnlockVaultDialogProps) {
  const t = texts.dialogs.unlock;
  const field = useRef<HTMLInputElement>(null);
  const unlock = useUnlockVault();
  const { data: vault } = useQuery(vaultStatusQuery);
  const [error, setError] = useState<string>();

  const submit = () => {
    const input = field.current;
    if (!input || unlock.isPending) return;
    const masterPassword = input.value;
    input.value = "";
    setError(undefined);
    unlock
      .mutateAsync(masterPassword)
      .then(onUnlocked)
      .catch((failure: unknown) => {
        setError(presentFailure(failure).message);
        field.current?.focus();
      });
  };

  return (
    <Dialog open onClose={onClose} label={t.title} width="small" centered onSubmit={submit}>
      <span className={styles.vaultTile} aria-hidden="true">
        <LockIcon size="vault" />
      </span>
      <h2 className={styles.centeredTitle}>{t.title}</h2>
      {vault?.path && <p className={styles.path}>{visibleText(vault.path)}</p>}
      <div className={styles.unlockField}>
        <TextField
          ref={field}
          label={t.field}
          type="password"
          autoComplete="current-password"
          message={error ? { tone: "error", text: error } : undefined}
          shake={Boolean(error)}
        />
      </div>
      <p className={styles.hint}>{t.hint}</p>
      <div className={styles.halves}>
        <Button onClick={onClose}>{t.withoutSecrets}</Button>
        <Button variant="primary" type="submit" loading={unlock.isPending}>
          {t.unlock}
        </Button>
      </div>
    </Dialog>
  );
}
