import { plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { LockIcon } from "./icons/LockIcon";
import styles from "./VaultStatus.module.css";
import { Tooltip } from "./Tooltip";
import { visibleText } from "./visibleText";

/** The vault as the sidebar footer shows it (ambiguity 20). */
export type VaultFooterState =
  { backend: "keychain" } | { backend: "file"; locked: boolean; fileName: string };

interface VaultStatusProps {
  vault: VaultFooterState;
  nodeCount: number;
  /** Opens the unlock dialog; only offered for a locked file vault. */
  onUnlock: () => void;
}

/**
 * The footer of the sidebar (inventario §5.1, ambiguity 20): "Portachiavi di
 * sistema" without state for the Keychain; for the file vault its state,
 * clickable to unlock while locked, with the file name in the tooltip.
 */
export function VaultStatus({ vault, nodeCount, onUnlock }: VaultStatusProps) {
  const t = texts.components.vault;
  const count = <span className={styles.count}>{plural(nodeCount, t.nodes)}</span>;
  if (vault.backend === "keychain") {
    return (
      <footer className={styles.footer}>
        <span className={styles.ok}>
          <LockIcon size="vault" />
        </span>
        <span>{t.keychain}</span>
        {count}
      </footer>
    );
  }
  const label = vault.locked ? t.fileLocked : t.fileUnlocked;
  return (
    <footer className={styles.footer}>
      <Tooltip content={visibleText(vault.fileName)}>
        {vault.locked ? (
          <button type="button" className={styles.unlock} onClick={onUnlock}>
            <LockIcon size="vault" />
            <span>{label}</span>
          </button>
        ) : (
          <span className={styles.state} tabIndex={0}>
            <span className={styles.ok}>
              <LockIcon open size="vault" />
            </span>
            <span>{label}</span>
          </span>
        )}
      </Tooltip>
      {count}
    </footer>
  );
}
