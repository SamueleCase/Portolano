import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import { LockIcon } from "./icons/LockIcon";
import styles from "./SecretBadge.module.css";

/**
 * What the frontend knows of a secret: never its value (SPEC-UI §1.1).
 * `staged` is a secret set in the editor and not saved yet (ambiguity 14).
 */
export type SecretState = "present" | "absent" | "staged" | "locked";

/** Where present secrets are kept: the badge names it (inventario §4.9). */
export type VaultBackend = "keychain" | "file";

interface SecretBadgeProps {
  state: SecretState;
  backend: VaultBackend;
  /**
   * `compact`: the smaller badge in a row ("password assente");
   * `text`: just the colored words, in the grids of the selector.
   */
  variant?: "pill" | "compact" | "text";
}

function label(state: SecretState, backend: VaultBackend, compact: boolean): string {
  const badge = texts.components.secretBadge;
  switch (state) {
    case "present":
      return badge.present[backend];
    case "absent":
      return compact ? badge.absentPassword : badge.absent;
    case "staged":
      return badge.staged;
    case "locked":
      return badge.locked;
  }
}

/**
 * The state of a secret, by color, border, padlock and words (inventario
 * §4.9). With the file vault locked it says "Vault bloccato", never "Assente".
 */
export function SecretBadge({ state, backend, variant = "pill" }: SecretBadgeProps) {
  const text = label(state, backend, variant === "compact");
  if (variant === "text") {
    return <span className={classNames(styles.text, styles[`${state}Text`])}>{text}</span>;
  }
  return (
    <span
      className={classNames(styles.badge, styles[state], variant === "compact" && styles.compact)}
    >
      {variant === "pill" && <LockIcon open={state === "absent"} />}
      {text}
    </span>
  );
}
