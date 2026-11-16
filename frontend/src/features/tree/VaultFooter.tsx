import { useQuery } from "@tanstack/react-query";

import { useWindowActions } from "../../app/windowActions";
import { LockIcon } from "../../components/icons/LockIcon";
import { Tooltip } from "../../components/Tooltip";
import { type VaultFooterState, VaultStatus } from "../../components/VaultStatus";
import { plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { vaultStatusQuery } from "../../ipc/queries";
import styles from "./Sidebar.module.css";

/** The file name of a path, for the tooltip of the footer. */
function fileName(path: string | null): string {
  return path?.split("/").pop() ?? "";
}

/**
 * The footer of the sidebar (ambiguity 20): the system keychain without a
 * state; the file vault with its state, which opens the unlock dialog while
 * locked (or the creation of the vault, if its file does not exist yet).
 * Without that dialog mounted the state is shown but not clickable, with the
 * reason in its tooltip (DEC-041.6).
 */
export function VaultFooter({ nodeCount }: { nodeCount: number | undefined }) {
  const actions = useWindowActions();
  const vault = useQuery(vaultStatusQuery).data;
  const missing = vault?.exists === false;
  const open = missing ? actions.createFileVault : actions.unlockVault;
  const offered = actions.offers(missing ? "createFileVault" : "unlockVault");
  if (vault?.kind === "file" && vault.locked && !offered) {
    const t = texts.components.vault;
    return (
      <footer className={styles.vaultFooter}>
        <Tooltip content={`${fileName(vault.path)} · ${texts.app.reasons.unavailable}`}>
          <span className={styles.vaultState} tabIndex={0}>
            <LockIcon size="vault" />
            <span>{t.fileLocked}</span>
          </span>
        </Tooltip>
        {nodeCount !== undefined && (
          <span className={styles.vaultCount}>{plural(nodeCount, t.nodes)}</span>
        )}
      </footer>
    );
  }
  const state: VaultFooterState =
    vault?.kind === "file"
      ? { backend: "file", locked: vault.locked, fileName: fileName(vault.path) }
      : { backend: "keychain" };
  return <VaultStatus vault={state} nodeCount={nodeCount} onUnlock={open} />;
}
