import type { Ref } from "react";

import type { SecretFieldView, SecretRequestView_Serialize } from "../../bindings";
import { Card } from "../../components/Card";
import { KeyValueGrid, type KeyValueRow } from "../../components/KeyValueGrid";
import { ProgressBar } from "../../components/ProgressBar";
import { TextField } from "../../components/TextField";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { MaskedKeyField } from "../dialogs/MaskedKeyField";
import { readableImportPath } from "./importPath";
import styles from "./ImportWizard.module.css";

/**
 * The field of a request: an input for a password, a text area for a
 * private key. Both are read through `value` only, so the wizard holds one
 * ref for either (the same trick as the secret dialog).
 */
export type SecretFieldElement = HTMLInputElement & HTMLTextAreaElement;

interface PasswordStepProps {
  request: SecretRequestView_Serialize;
  /** The index of `request`, from 0. */
  position: number;
  total: number;
  entered: number;
  skipped: number;
  /**
   * The field of the password, or of the private key. It is uncontrolled:
   * the value is read from the element when the user goes on, never kept in
   * React state.
   */
  fieldRef: Ref<SecretFieldElement>;
}

function fieldLabel(field: SecretFieldView): string {
  const { fields } = texts.importWizard.passwords;
  return field.kind === "property"
    ? interpolate(fields.property, { name: field.name })
    : fields[field.kind];
}

/**
 * Step 3 of the wizard (inventario §5.13): one request at a time, "N / M",
 * the node, its user and the field. The password goes to the answers of the
 * wizard when the user goes on, and the field is mounted again empty for the
 * next request (`key` of the caller). A private key goes in the masked area
 * of the secret dialog, which keeps its line breaks (DEC-044.1); Enter there
 * starts a new line, and "Salva e continua" goes on.
 */
export function PasswordStep({
  request,
  position,
  total,
  entered,
  skipped,
  fieldRef,
}: PasswordStepProps) {
  const { passwords } = texts.importWizard;
  const counter = interpolate(passwords.position, { position: position + 1, total });
  const rows: KeyValueRow[] = [
    {
      key: "node",
      label: passwords.node,
      value: readableImportPath(request.nodePath),
      mono: true,
    },
    ...(request.username === null
      ? []
      : [{ key: "user", label: passwords.user, value: request.username, mono: true }]),
    { key: "field", label: passwords.field, value: fieldLabel(request.field) },
  ];
  return (
    <div className={styles.step}>
      <div className={styles.progressHead}>
        <span className={styles.counter}>{counter}</span>
        <span className={styles.muted}>
          {plural(entered, passwords.entered)} · {plural(skipped, passwords.skipped)}
        </span>
      </div>
      <ProgressBar
        label={passwords.progress}
        value={position + 1}
        max={total}
        valueText={counter}
      />
      <Card className={styles.requestCard}>
        <KeyValueGrid rows={rows} width="import" />
      </Card>
      {request.field.kind === "privateKey" ? (
        <MaskedKeyField ref={fieldRef} label={fieldLabel(request.field)} autoFocus />
      ) : (
        <TextField
          ref={fieldRef}
          label={fieldLabel(request.field)}
          type="password"
          autoComplete="off"
          autoFocus
        />
      )}
      <p className={styles.muted}>{passwords.explanation}</p>
    </div>
  );
}
