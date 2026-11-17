import type { Ref } from "react";

import type { FieldMessage } from "../../components/FormField";
import { TextArea } from "../../components/TextArea";
import styles from "./MaskedKeyField.module.css";

interface MaskedKeyFieldProps {
  label: string;
  message?: FieldMessage | undefined;
  onInput?: (() => void) | undefined;
  autoFocus?: boolean | undefined;
  /**
   * The area, read by the caller when the key is sent. It is uncontrolled:
   * the key is never React state, and the caller empties the area (or
   * mounts a new one) once the value is sent (SPEC-UI §1.2).
   */
  ref?: Ref<HTMLTextAreaElement>;
}

/**
 * The field of a private key (DEC-038.5): a password field drops the line
 * breaks of a key, so the key goes in a text area that shows dots as a
 * password field does, without autocomplete, spelling or a way to copy the
 * key out. Shared by the secret dialog and the import wizard (DEC-044.1).
 */
export function MaskedKeyField({ label, message, onInput, autoFocus, ref }: MaskedKeyFieldProps) {
  return (
    <TextArea
      ref={ref ?? null}
      label={label}
      mono
      rows={6}
      autoComplete="off"
      autoCorrect="off"
      autoCapitalize="off"
      spellCheck={false}
      autoFocus={autoFocus}
      className={styles.maskedKey}
      message={message}
      onInput={onInput}
      // Dots on screen, and no way to copy the key out of the area, as
      // a password field.
      onCopy={(event) => event.preventDefault()}
      onCut={(event) => event.preventDefault()}
    />
  );
}
