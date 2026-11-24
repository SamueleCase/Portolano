import { GlassPanel } from "../components/GlassPanel";
import { Logo } from "../components/Logo";
import { isConnection } from "../components/nodeKind";
import { visibleText } from "../components/visibleText";
import { useSelectedEntry } from "../features/tree/useTreeIndex";
import { texts } from "../i18n/it";
import { GuardedButton } from "./GuardedButton";
import styles from "./Toolbar.module.css";
import { useWindowActions } from "./windowActions";

/**
 * The bar of the window (inventario §5.1, DEC-050): the room for the native
 * traffic lights (no fake ones, §8), the logo, the document and "Connetti";
 * the creation, the import, the validation and the preferences live in the
 * sidebar. The whole bar drags the window (`data-tauri-drag-region`), its
 * button excepted.
 */
export function Toolbar({ documentName }: { documentName: string }) {
  const t = texts.app.toolbar;
  const { reasons, shortcuts } = texts.app;
  const actions = useWindowActions();
  const selection = useSelectedEntry();
  const connectable = selection !== undefined && isConnection(selection.kind);

  return (
    <GlassPanel as="header" aria-label={t.label} className={styles.bar} data-tauri-drag-region="">
      <span className={styles.trafficLights} data-tauri-drag-region="" />
      <Logo />
      <span className={styles.document} data-tauri-drag-region="">
        {visibleText(documentName)}
      </span>
      <GuardedButton
        className={styles.connect}
        variant="primary"
        size="wide"
        reason={connectable ? undefined : reasons.notConnection}
        hint={t.connect}
        shortcut={shortcuts.connect}
        onClick={() => selection && actions.connect(selection.node.id)}
      >
        {t.connect}
      </GuardedButton>
    </GlassPanel>
  );
}
