import type { ButtonHTMLAttributes, MouseEvent, Ref } from "react";

import { texts } from "../i18n/it";
import styles from "./Button.module.css";
import { classNames } from "./classNames";
import { Spinner } from "./Spinner";
import { VisuallyHidden } from "./VisuallyHidden";

/** The variants of inventario §4.1. */
export type ButtonVariant =
  "primary" | "secondary" | "toolbar" | "ghost" | "add" | "danger" | "danger-text" | "small";

interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "disabled"> {
  variant?: ButtonVariant;
  /** `wide`: primary of the toolbar and the editor; `large`: the empty state. */
  size?: "normal" | "wide" | "large";
  /**
   * Shown as in the DS and announced with `aria-disabled` (ambiguity 7): the
   * button stays focusable, so its label can still be read.
   */
  disabled?: boolean;
  /** A command is running: same label, a spinner, and clicks are ignored. */
  loading?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

const variantClass: Record<ButtonVariant, string | undefined> = {
  primary: styles.primary,
  secondary: styles.secondary,
  toolbar: styles.toolbar,
  ghost: styles.ghost,
  add: styles.add,
  danger: styles.danger,
  "danger-text": styles.dangerText,
  small: styles.small,
};

/** A button of the interface. */
export function Button({
  variant = "secondary",
  size = "normal",
  disabled = false,
  loading = false,
  type = "button",
  className,
  onClick,
  children,
  ...rest
}: ButtonProps) {
  const inert = disabled || loading;
  const click = (event: MouseEvent<HTMLButtonElement>) => {
    if (inert) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  };
  return (
    <button
      {...rest}
      type={type}
      className={classNames(
        styles.button,
        variantClass[variant],
        size !== "normal" && styles[size],
        disabled && styles.disabled,
        className,
      )}
      aria-disabled={disabled || undefined}
      aria-busy={loading || undefined}
      onClick={click}
    >
      {loading && <Spinner />}
      {children}
      {loading && (
        <>
          {" "}
          <VisuallyHidden>{texts.components.busy}</VisuallyHidden>
        </>
      )}
    </button>
  );
}
