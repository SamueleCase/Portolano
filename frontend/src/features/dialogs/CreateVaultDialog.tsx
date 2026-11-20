import { useQuery } from "@tanstack/react-query";
import { useRef, useState } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import type { FieldMessage } from "../../components/FormField";
import { TextField } from "../../components/TextField";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useCreateFileVault } from "../../ipc/mutations";
import { vaultStatusQuery } from "../../ipc/queries";
import styles from "./Dialogs.module.css";

/** The shortest master password of a new vault (DEC-024.7, DEC-026.6). */
const MIN_MASTER_PASSWORD_CHARS = 12;

interface CreateVaultDialogProps {
  open: boolean;
  onClose: () => void;
  /** The vault file exists now, open with the master password typed. */
  onCreated: () => void;
}

type Problem = "tooShort" | "mismatch";

/**
 * The creation of the file vault (inventario §5.17): a new master password
 * of at least 12 characters, typed twice, sent to `create_file_vault`.
 */
export function CreateVaultDialog(props: CreateVaultDialogProps) {
  return props.open ? <OpenCreateVaultDialog {...props} /> : null;
}

function OpenCreateVaultDialog({ onClose, onCreated }: CreateVaultDialogProps) {
  const t = texts.dialogs.create;
  const first = useRef<HTMLInputElement>(null);
  const second = useRef<HTMLInputElement>(null);
  const create = useCreateFileVault();
  const { data: vault } = useQuery(vaultStatusQuery);
  const [problem, setProblem] = useState<Problem>();
  const [matching, setMatching] = useState(false);
  const [error, setError] = useState<string>();
  // Each master password sent gets new fields: the undo of the old ones would give it back.
  const [generation, setGeneration] = useState(0);

  const compare = () => {
    const typed = first.current?.value ?? "";
    setMatching(typed !== "" && typed === second.current?.value);
    setProblem(undefined);
  };

  const submit = () => {
    const password = first.current;
    const again = second.current;
    if (!password || !again || create.isPending) return;
    // Characters, not UTF-16 units, as the backend counts them.
    if ([...password.value].length < MIN_MASTER_PASSWORD_CHARS) {
      setProblem("tooShort");
      return;
    }
    if (password.value !== again.value) {
      setProblem("mismatch");
      return;
    }
    const masterPassword = password.value;
    password.value = "";
    again.value = "";
    setGeneration((current) => current + 1);
    setMatching(false);
    setError(undefined);
    create
      .mutateAsync(masterPassword)
      .then(onCreated)
      .catch((failure: unknown) => {
        setError(presentFailure(failure).message);
        // The new field, in place of the one whose value was sent.
        first.current?.focus();
      });
  };

  const firstMessage: FieldMessage | undefined =
    problem === "tooShort"
      ? { tone: "error", text: t.tooShort }
      : error
        ? { tone: "error", text: error }
        : undefined;
  const secondMessage: FieldMessage | undefined =
    problem === "mismatch"
      ? { tone: "error", text: t.mismatch }
      : matching
        ? { tone: "ok", text: t.match }
        : undefined;

  return (
    <Dialog
      open
      onClose={onClose}
      title={t.title}
      subtitle={vault?.path ? visibleText(vault.path) : undefined}
      width="small"
      onSubmit={submit}
      footer={
        <>
          <Button onClick={onClose}>{texts.dialogs.cancel}</Button>
          <Button variant="primary" type="submit" loading={create.isPending}>
            {t.create}
          </Button>
        </>
      }
    >
      <p className={styles.hint}>{t.text}</p>
      <TextField
        key={`password-${generation}`}
        ref={first}
        label={t.field}
        type="password"
        autoComplete="new-password"
        message={firstMessage}
        onInput={compare}
      />
      <TextField
        key={`confirmation-${generation}`}
        ref={second}
        label={t.confirm}
        type="password"
        autoComplete="new-password"
        message={secondMessage}
        shake={problem === "mismatch"}
        onInput={compare}
      />
    </Dialog>
  );
}
