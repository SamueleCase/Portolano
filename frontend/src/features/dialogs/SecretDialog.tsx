import { useRef, useState } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import type { FieldMessage } from "../../components/FormField";
import type { VaultBackend } from "../../components/SecretBadge";
import { TextArea } from "../../components/TextArea";
import { TextField } from "../../components/TextField";
import type { SecretSlotInput } from "../../bindings";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./Dialogs.module.css";

interface SecretDialogProps {
  open: boolean;
  /** The secret to type: its kind names the title and the field. */
  slot: SecretSlotInput;
  /** The node, as the subtitle shows it ("Clienti / Admin ACME"). */
  nodeLabel: string;
  /** "Sostituisci…" rather than "Imposta…". */
  replacing: boolean;
  /**
   * `stage`: the editor keeps the value until "Salva" (button "Imposta");
   * `save`: the caller saves it at once (button "Salva nel portachiavi").
   */
  mode: "stage" | "save";
  backend: VaultBackend;
  busy?: boolean;
  /** The error of the save, under the field. */
  error?: string | undefined;
  /**
   * The value typed. The dialog empties its fields right after the call:
   * the value lives only in the field and in this call (SPEC-UI §1.2).
   */
  onConfirm: (value: string) => void;
  onClose: () => void;
}

type Problem = "empty" | "mismatch";

/** The title of the dialog for `slot`. */
function secretTitle(slot: SecretSlotInput, replacing: boolean): string {
  const titles = replacing ? texts.dialogs.secret.replaceTitles : texts.dialogs.secret.titles;
  return slot.kind === "property"
    ? interpolate(titles.property, { name: slot.name })
    : titles[slot.kind];
}

/**
 * The dialog that asks for a password, a passphrase, a private key or a
 * secret property (inventario §5.9 with its corrections). The fields are
 * uncontrolled: what is typed never becomes React state; only whether the
 * two values match does. A private key is pasted in a masked text area,
 * since a password field drops its line breaks.
 */
export function SecretDialog(props: SecretDialogProps) {
  // Mounted only while open: every opening starts with empty fields.
  return props.open ? <OpenSecretDialog {...props} /> : null;
}

function OpenSecretDialog({
  slot,
  nodeLabel,
  replacing,
  mode,
  backend,
  busy = false,
  error,
  onConfirm,
  onClose,
}: SecretDialogProps) {
  const t = texts.dialogs.secret;
  const value = useRef<HTMLInputElement & HTMLTextAreaElement>(null);
  const confirmation = useRef<HTMLInputElement>(null);
  const [problem, setProblem] = useState<Problem>();
  const [matching, setMatching] = useState(false);
  // Each value sent gets new fields: the undo of the old ones would give it back.
  const [generation, setGeneration] = useState(0);
  const isKey = slot.kind === "privateKey";

  const compare = () => {
    const typed = value.current?.value ?? "";
    const again = confirmation.current?.value ?? "";
    setMatching(typed !== "" && typed === again);
    if (problem === "mismatch" && typed === again) setProblem(undefined);
    if (problem === "empty" && typed !== "") setProblem(undefined);
  };

  const submit = () => {
    const field = value.current;
    if (!field || busy) return;
    if (field.value === "") {
      setProblem("empty");
      return;
    }
    if (!isKey && field.value !== confirmation.current?.value) {
      setProblem("mismatch");
      return;
    }
    const typed = field.value;
    field.value = "";
    if (confirmation.current) confirmation.current.value = "";
    setMatching(false);
    setGeneration((current) => current + 1);
    onConfirm(typed);
  };

  const fieldMessage: FieldMessage | undefined =
    problem === "empty"
      ? { tone: "error", text: t.empty }
      : error
        ? { tone: "error", text: error }
        : undefined;
  // A private key has no confirmation: the kinds left name the two messages.
  const compared = slot.kind === "privateKey" ? "property" : slot.kind;
  const confirmMessage: FieldMessage | undefined =
    problem === "mismatch"
      ? { tone: "error", text: t.mismatch[compared] }
      : matching
        ? { tone: "ok", text: t.match[compared] }
        : undefined;
  const vault = texts.dialogs.vaultNames[backend];
  const label = t.fields[slot.kind];

  return (
    <Dialog
      open
      onClose={onClose}
      title={secretTitle(slot, replacing)}
      subtitle={nodeLabel}
      width="small"
      onSubmit={submit}
      footer={
        <>
          <Button onClick={onClose}>{texts.dialogs.cancel}</Button>
          <Button variant="primary" type="submit" loading={busy}>
            {mode === "stage" ? t.stage : t.save[backend]}
          </Button>
        </>
      }
    >
      {isKey ? (
        <TextArea
          key={generation}
          ref={value}
          label={label}
          mono
          rows={6}
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          className={styles.maskedKey}
          message={fieldMessage}
          onInput={compare}
          // Dots on screen, and no way to copy the key out of the area, as
          // a password field.
          onCopy={(event) => event.preventDefault()}
          onCut={(event) => event.preventDefault()}
        />
      ) : (
        <>
          <TextField
            key={`value-${generation}`}
            ref={value}
            label={label}
            type="password"
            autoComplete="new-password"
            message={fieldMessage}
            onInput={compare}
          />
          <TextField
            key={`confirmation-${generation}`}
            ref={confirmation}
            label={t.confirm}
            type="password"
            autoComplete="new-password"
            message={confirmMessage}
            shake={problem === "mismatch"}
            onInput={compare}
          />
        </>
      )}
      <p className={styles.hint}>
        {isKey && `${t.keyHint} `}
        {interpolate(mode === "stage" ? t.staged : t.saved, { vault })}
      </p>
    </Dialog>
  );
}
