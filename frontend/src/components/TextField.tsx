import { type InputHTMLAttributes, type Ref, useId } from "react";

import { classNames } from "./classNames";
import fieldStyles from "./field.module.css";
import { type FieldMessage, FormField } from "./FormField";
import styles from "./TextField.module.css";

interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  label: string;
  /** Hosts, ports, users, paths, commands and options (inventario §4.2). */
  mono?: boolean;
  /** `search`: the sidebar search; `numeric`: the 56 px keepalive and resolution fields. */
  variant?: "normal" | "search" | "numeric";
  message?: FieldMessage | undefined;
  /** Shakes the field when its error appears (mismatching passwords). */
  shake?: boolean;
  hideLabel?: boolean | undefined;
  className?: string | undefined;
  ref?: Ref<HTMLInputElement>;
}

/**
 * A text field with its label and message. With `type="password"` the field
 * shows dots and never offers to show the password (inventario §4.2).
 */
export function TextField({
  label,
  mono = false,
  variant = "normal",
  message,
  shake = false,
  hideLabel = variant === "search",
  className,
  id,
  type = "text",
  ...rest
}: TextFieldProps) {
  const generatedId = useId();
  const controlId = id ?? generatedId;
  const messageId = `${controlId}-message`;
  return (
    <FormField
      label={label}
      controlId={controlId}
      messageId={messageId}
      message={message}
      hideLabel={hideLabel}
      className={classNames(variant === "numeric" && styles.numericField, className)}
    >
      <input
        {...rest}
        id={controlId}
        type={type}
        spellCheck={rest.spellCheck ?? (mono ? false : undefined)}
        aria-invalid={message?.tone === "error" || undefined}
        aria-describedby={message ? messageId : rest["aria-describedby"]}
        className={classNames(
          fieldStyles.control,
          mono && fieldStyles.mono,
          type === "password" && styles.password,
          variant === "search" && styles.search,
          variant === "numeric" && styles.numeric,
          message?.tone === "error" && fieldStyles.invalid,
          message?.tone === "error" && shake && fieldStyles.shake,
          message?.tone === "ok" && fieldStyles.confirmed,
        )}
      />
    </FormField>
  );
}
