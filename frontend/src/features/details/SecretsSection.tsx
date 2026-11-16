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
  const setReason = !actions.offers("password")
    ? texts.app.reasons.unavailable
    : details.vaultLocked
      ? texts.components.secretBadge.locked
      : undefined;

  return (
    <section className={styles.section} aria-label={t.heading}>
      <GroupHeader>{t.heading}</GroupHeader>
      {entries.map((entry) => {
        const set = entry.state !== undefined;
        return (
          <div key={entry.label} className={styles.secretRow} role="group" aria-label={entry.label}>
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
                <Button variant="danger-text" onClick={() => setClearing(entry)}>
                  {t.clear}
                </Button>
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
