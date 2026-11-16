import { useQuery } from "@tanstack/react-query";

import { useWindowActions } from "../../app/windowActions";
import { type VaultFooterState, VaultStatus } from "../../components/VaultStatus";
import { vaultStatusQuery } from "../../ipc/queries";

/** The file name of a path, for the tooltip of the footer. */
function fileName(path: string | null): string {
  return path?.split("/").pop() ?? "";
}

/**
 * The footer of the sidebar (ambiguity 20): the system keychain without a
 * state; the file vault with its state, which opens the unlock dialog while
 * locked (or the creation of the vault, if its file does not exist yet).
 */
export function VaultFooter({ nodeCount }: { nodeCount: number }) {
  const { unlockVault, createFileVault } = useWindowActions();
  const vault = useQuery(vaultStatusQuery).data;
  const state: VaultFooterState =
    vault?.kind === "file"
      ? { backend: "file", locked: vault.locked, fileName: fileName(vault.path) }
      : { backend: "keychain" };
  return (
    <VaultStatus
      vault={state}
      nodeCount={nodeCount}
      onUnlock={vault?.exists === false ? createFileVault : unlockVault}
    />
  );
}
