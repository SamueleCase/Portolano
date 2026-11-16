import { useState } from "react";

import { GuardedButton } from "../../app/GuardedButton";
import { useWindowActions } from "../../app/windowActions";
import type { NodeDetailsView_Serialize } from "../../bindings";
import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { GroupHeader } from "../../components/GroupHeader";
import { SecretBadge, type VaultBackend } from "../../components/SecretBadge";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./Details.module.css";
import { type SecretEntry, secretEntries } from "./secretSlots";

interface SecretsSectionProps {
  details: NodeDetailsView_Serialize;
  backend: VaultBackend;
}

/**
 * The secrets of the node itself (inventario §5.2, §4.26 `SecretRow`): their
 * state, "Imposta…" or "Sostituisci…" through the password dialog of the hook
 * point `password` (written at once with `set_secret`), and "Cancella", which
 * asks first and then calls `clear_secret`. With the vault locked nothing
 * can be set until it is unlocked.
 */
export function SecretsSection({ details, backend }: SecretsSectionProps) {
  const t = texts.details.secrets;
  const actions = useWindowActions();
  const [clearing, setClearing] = useState<SecretEntry>();
  const entries = secretEntries(details);
  if (entries.length === 0) return null;
  const node = { id: details.id, path: details.path };
  // With the vault locked a secret can be set or cleared once it is unlocked:
  // the actions ask for it first when the unlock dialog is there (DEC-041.2).
  const locked =
    details.vaultLocked && !actions.offers("unlockVault") && !actions.offers("createFileVault");
  const lockedReason = locked ? texts.components.secretBadge.locked : undefined;
  const setReason = actions.offers("password") ? lockedReason : texts.app.reasons.unavailable;

  return (
    <section className={styles.section} aria-label={t.heading}>
      <GroupHeader>{t.heading}</GroupHeader>
      {entries.map((entry) => {
        const set = entry.state !== undefined;
        const key =
          entry.slot.kind === "property" ? `property:${entry.slot.name}` : entry.slot.kind;
        return (
          <div
            key={key}
            className={styles.secretRow}
            role="group"
            aria-label={visibleText(entry.label)}
          >
            <span className={styles.secretLabel}>{visibleText(entry.label)}</span>
            <SecretBadge state={entry.state ?? "absent"} backend={backend} />
            <span className={styles.secretActions}>
              <GuardedButton
                variant="ghost"
                reason={setReason}
                onClick={() => actions.requestPassword(node, entry.slot)}
              >
                {entry.state === "present" ? t.replace : t.set}
              </GuardedButton>
              {set && (
                <GuardedButton
                  variant="danger-text"
                  reason={lockedReason}
                  onClick={() => setClearing(entry)}
                >
                  {t.clear}
                </GuardedButton>
              )}
            </span>
          </div>
        );
      })}
      <Dialog
        open={clearing !== undefined}
        onClose={() => setClearing(undefined)}
        title={interpolate(t.clearTitle, { slot: clearing?.label ?? "", name: details.name })}
        footer={
          <>
            <Button onClick={() => setClearing(undefined)}>{t.cancel}</Button>
            <Button
              variant="danger"
              onClick={() => {
                if (clearing) actions.clearSecret(details.id, clearing.slot);
                setClearing(undefined);
              }}
            >
              {t.clear}
            </Button>
          </>
        }
      >
        <p className={styles.dialogText}>{t.clearText}</p>
      </Dialog>
    </section>
  );
}
