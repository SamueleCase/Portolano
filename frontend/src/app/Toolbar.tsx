import { useState } from "react";

import { GlassPanel } from "../components/GlassPanel";
import { Logo } from "../components/Logo";
import { isConnection } from "../components/nodeKind";
import { visibleText } from "../components/visibleText";
import { useSelectedEntry } from "../features/tree/useTreeIndex";
import { texts } from "../i18n/it";
import { useWorkspace } from "../state/workspace";
import { GuardedButton } from "./GuardedButton";
import { NewConnectionMenu } from "./NewConnectionMenu";
import styles from "./Toolbar.module.css";
import { useWindowActions } from "./windowActions";

/**
 * The toolbar (inventario §5.1): the room for the native traffic lights (no
 * fake ones, §8), the logo and the document, the creation buttons, the
 * functions of the hook points and "Connetti". The whole bar drags the
 * window (`data-tauri-drag-region`), its buttons excepted.
 */
export function Toolbar({ documentName }: { documentName: string }) {
  const t = texts.app.toolbar;
  const { reasons, shortcuts } = texts.app;
  const actions = useWindowActions();
  const selection = useSelectedEntry();
  const menuOpen = useWorkspace((state) => state.connectionMenuOpen);
  const setMenu = useWorkspace((state) => state.setConnectionMenu);
  const [anchor, setAnchor] = useState<HTMLButtonElement | null>(null);
  const offered = (feature: Parameters<typeof actions.offers>[0]) =>
    actions.offers(feature) ? undefined : reasons.unavailable;
  const connectable = selection !== undefined && isConnection(selection.kind);

  return (
    <GlassPanel as="header" className={styles.bar} data-tauri-drag-region="">
      <span className={styles.trafficLights} data-tauri-drag-region="" />
      <Logo />
      <span className={styles.document} data-tauri-drag-region="">
        {visibleText(documentName)}
      </span>
      <nav aria-label={t.label} className={styles.actions}>
        <GuardedButton
          ref={setAnchor}
          variant="toolbar"
          aria-haspopup="menu"
          aria-expanded={menuOpen}
          reason={offered("editor")}
          hint={t.newConnectionMenu}
          shortcut={shortcuts.newConnection}
          onClick={() => setMenu(true)}
        >
          {t.newConnection}
        </GuardedButton>
        <GuardedButton
          variant="toolbar"
          reason={offered("editor")}
          onClick={() => actions.create("folder")}
        >
          {t.newFolder}
        </GuardedButton>
        <GuardedButton
          variant="toolbar"
          reason={offered("editor")}
          onClick={() => actions.create("credential")}
        >
          {t.newCredential}
        </GuardedButton>
        <span className={styles.separator} aria-hidden="true" />
        <GuardedButton variant="ghost" reason={offered("import")} onClick={actions.openImport}>
          {t.import}
        </GuardedButton>
        <GuardedButton
          variant="ghost"
          reason={offered("validation")}
          onClick={actions.openValidation}
        >
          {t.validate}
        </GuardedButton>
        <GuardedButton
          variant="ghost"
          reason={offered("preferences")}
          onClick={actions.openPreferences}
        >
          {t.preferences}
        </GuardedButton>
        <GuardedButton
          variant="primary"
          size="wide"
          reason={connectable ? undefined : reasons.notConnection}
          hint={t.connect}
          shortcut={shortcuts.connect}
          onClick={() => selection && actions.connect(selection.node.id)}
        >
          {t.connect}
        </GuardedButton>
      </nav>
      {menuOpen && anchor && <NewConnectionMenu anchor={anchor} onClose={() => setMenu(false)} />}
    </GlassPanel>
  );
}
