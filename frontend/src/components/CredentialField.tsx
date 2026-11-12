import { useId } from "react";

import { texts } from "../i18n/it";
import styles from "./CredentialField.module.css";
import { LockIcon } from "./icons/LockIcon";
import { ModeChip } from "./ModeChip";
import { SecretBadge, type SecretState, type VaultBackend } from "./SecretBadge";
import { visibleText } from "./visibleText";

interface CredentialFieldProps {
  /** "Credenziale", "Credenziale predefinita", "Credenziale del gateway". */
  label: string;
  /** The mode, such as "Eredita" or "Da una credenziale". */
  mode: string;
  /** Where it comes from, such as "Produzione → deploy" (untrusted names). */
  origin?: string | undefined;
  /** The user, in mono, for RDP (`DOMINIO\utente`). */
  user?: string | undefined;
  /** The password of the credential, if the mode has one. */
  password?: SecretState | undefined;
  backend: VaultBackend;
  /** Opens the credential selector. */
  onChange: () => void;
  /** Goes to the credential whose password is missing. */
  onShowCredential?: (() => void) | undefined;
}

/** The field that shows a credential and opens its selector (inventario §4.23). */
export function CredentialField({
  label,
  mode,
  origin,
  user,
  password,
  backend,
  onChange,
  onShowCredential,
}: CredentialFieldProps) {
  const labelId = useId();
  const absent = password === "absent";
  return (
    <div className={styles.field} role="group" aria-labelledby={labelId}>
      <span id={labelId} className={styles.label}>
        {label}
      </span>
      <div className={styles.box}>
        <ModeChip>{mode}</ModeChip>
        {user && <span className={styles.user}>{visibleText(user)}</span>}
        {origin && <span className={styles.origin}>{visibleText(origin)}</span>}
        {password === "present" && (
          <span
            className={styles.present}
            role="img"
            aria-label={texts.components.credentialField.passwordPresent}
          >
            <LockIcon />
          </span>
        )}
        {absent && onShowCredential && (
          <button type="button" className={styles.badgeButton} onClick={onShowCredential}>
            <SecretBadge state="absent" backend={backend} variant="compact" />
          </button>
        )}
        {password && password !== "present" && !(absent && onShowCredential) && (
          <SecretBadge state={password} backend={backend} variant="compact" />
        )}
        <button type="button" className={styles.change} onClick={onChange}>
          {texts.components.credentialField.change}
        </button>
      </div>
    </div>
  );
}
