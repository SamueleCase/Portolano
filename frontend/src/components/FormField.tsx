import type { ReactNode } from "react";

import { classNames } from "./classNames";
import styles from "./FormField.module.css";

/** A message under a field: an error, or a confirmation such as matching passwords. */
export interface FieldMessage {
  tone: "error" | "ok";
  text: string;
}

interface FormFieldProps {
  label: string;
  /** The id of the control the label names. */
  controlId: string;
  /** The id of the message, for the control's `aria-describedby`. */
  messageId?: string | undefined;
  message?: FieldMessage | undefined;
  /** Keeps the label for assistive technologies only (search fields). */
  hideLabel?: boolean | undefined;
  className?: string | undefined;
  children: ReactNode;
}

/** A label 12 px muted above a control, with an optional message below (inventario §4.26). */
export function FormField({
  label,
  controlId,
  messageId,
  message,
  hideLabel = false,
  className,
  children,
}: FormFieldProps) {
  return (
    <div className={classNames(styles.field, className)}>
      <label htmlFor={controlId} className={classNames(styles.label, hideLabel && styles.hidden)}>
        {label}
      </label>
      {children}
      {message && (
        <span
          id={messageId}
          className={classNames(styles.message, message.tone === "ok" && styles.ok)}
          role={message.tone === "error" ? "alert" : undefined}
        >
          {message.text}
        </span>
      )}
    </div>
  );
}
