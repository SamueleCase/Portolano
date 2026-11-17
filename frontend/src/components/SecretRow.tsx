import { useId } from "react";

import { texts } from "../i18n/it";
import { Button } from "./Button";
import { SecretBadge, type SecretState, type VaultBackend } from "./SecretBadge";
import styles from "./SecretRow.module.css";
import { visibleText } from "./visibleText";

interface SecretRowProps {
  /** "Password", "Passphrase", or the name of a secret property. */
  label: string;
  state: SecretState;
  backend: VaultBackend;
  /** Opens the password dialog: "Imposta…" or "Sostituisci…". */
  onSet: () => void;
  /** Stages the removal of the secret, applied on Save (SPEC-UI §3.6). */
  onClear?: (() => void) | undefined;
  /**
   * The actions only stage a change applied on Save (the editor): with the
   * vault locked a saved secret can still be replaced or cleared, and the
   * vault is unlocked on Save (DEC-043.6).
   */
  appliesOnSave?: boolean;
  /** The id of a message about the secret (an error), read with its actions. */
  describedBy?: string | undefined;
}

/**
 * A secret of the editor (inventario §4.26): its state and its actions. A
 * present secret offers "Sostituisci…" and "Cancella", an absent one
 * "Imposta…". With the vault locked nothing can be set, unless the actions
 * apply on Save: the locked secret is then a saved one.
 */
export function SecretRow({
  label,
  state,
  backend,
  onSet,
  onClear,
  appliesOnSave = false,
  describedBy,
}: SecretRowProps) {
  const labelId = useId();
  const locked = state === "locked";
  const has = state === "present" || state === "staged" || (locked && appliesOnSave);
  return (
    <div className={styles.row} role="group" aria-labelledby={labelId}>
      <span id={labelId} className={styles.label}>
        {visibleText(label)}
      </span>
      <SecretBadge state={state} backend={backend} />
      <span className={styles.actions}>
        <Button
          variant="ghost"
          disabled={locked && !appliesOnSave}
          aria-describedby={describedBy}
          onClick={onSet}
        >
          {has ? texts.components.secretRow.replace : texts.components.secretRow.set}
        </Button>
        {has && onClear && (
          <Button variant="danger-text" aria-describedby={describedBy} onClick={onClear}>
            {texts.components.secretRow.clear}
          </Button>
        )}
      </span>
    </div>
  );
}
