import { useState } from "react";

import { GuardedButton } from "../../app/GuardedButton";
import { NewConnectionMenu } from "../../app/NewConnectionMenu";
import { useWindowActions } from "../../app/windowActions";
import { EmptyState } from "../../components/EmptyState";
import type { VaultBackend } from "../../components/SecretBadge";
import { texts } from "../../i18n/it";

/**
 * The empty document (inventario §5.1): the first connection or an import; the
 * sentence on the passwords names the vault of the configuration.
 */
export function EmptyDocument({ backend }: { backend: VaultBackend }) {
  const t = texts.firstRun.empty;
  const actions = useWindowActions();
  const [anchor, setAnchor] = useState<HTMLButtonElement | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const unavailable = texts.app.reasons.unavailable;
  return (
    <EmptyState
      title={t.title}
      text={t.text[backend]}
      actions={
        <>
          <GuardedButton
            ref={setAnchor}
            variant="primary"
            size="large"
            aria-haspopup="menu"
            aria-expanded={menuOpen}
            reason={actions.offers("editor") ? undefined : unavailable}
            onClick={() => setMenuOpen(true)}
          >
            {t.newConnection}
          </GuardedButton>
          <GuardedButton
            size="large"
            reason={actions.offers("import") ? undefined : unavailable}
            onClick={actions.openImport}
          >
            {t.import}
          </GuardedButton>
          {menuOpen && anchor && (
            <NewConnectionMenu anchor={anchor} onClose={() => setMenuOpen(false)} />
          )}
        </>
      }
    />
  );
}
