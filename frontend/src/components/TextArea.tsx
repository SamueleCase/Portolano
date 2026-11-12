import { type Ref, type TextareaHTMLAttributes, useId } from "react";

import { classNames } from "./classNames";
import fieldStyles from "./field.module.css";
import { type FieldMessage, FormField } from "./FormField";
import styles from "./TextArea.module.css";

interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label: string;
  mono?: boolean;
  message?: FieldMessage | undefined;
  className?: string | undefined;
  ref?: Ref<HTMLTextAreaElement>;
}

/** A text area with its label, such as the notes of a node (inventario §4.3: 52 px high). */
export function TextArea({ label, mono = false, message, className, id, ...rest }: TextAreaProps) {
  const generatedId = useId();
  const controlId = id ?? generatedId;
  const messageId = `${controlId}-message`;
  return (
    <FormField
      label={label}
      controlId={controlId}
      messageId={messageId}
      message={message}
      className={className}
    >
      <textarea
        {...rest}
        id={controlId}
        aria-invalid={message?.tone === "error" || undefined}
        aria-describedby={message ? messageId : undefined}
        className={classNames(
          fieldStyles.control,
          styles.area,
          mono && fieldStyles.mono,
          message?.tone === "error" && fieldStyles.invalid,
        )}
      />
    </FormField>
  );
}
