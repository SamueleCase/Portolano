import { useQueryClient } from "@tanstack/react-query";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";

import { vaultStatusQuery } from "../../ipc/queries";
import { CreateVaultDialog } from "./CreateVaultDialog";
import { UnlockVaultDialog } from "./UnlockVaultDialog";

/** What the caller of {@link useVaultUnlock} mounts and calls. */
export interface VaultUnlock {
  /**
   * Resolves `true` once the vault can be used: at once for the keychain or
   * an open file vault; otherwise after the unlock dialog, or the creation
   * dialog if the vault file does not exist yet. `false` if the user gives
   * up. Rejects only if the state of the vault cannot be read.
   */
  ensureUnlocked: () => Promise<boolean>;
  /** The dialog to render while it asks. */
  dialog: ReactNode;
}

/**
 * Asks for the master password when an action needs the file vault (the
 * "Salva" of the editor with secrets, a password set from the detail).
 */
export function useVaultUnlock(): VaultUnlock {
  const queryClient = useQueryClient();
  const [asking, setAsking] = useState<"unlock" | "create">();
  const resolve = useRef<((unlocked: boolean) => void) | undefined>(undefined);

  const finish = (unlocked: boolean) => {
    setAsking(undefined);
    resolve.current?.(unlocked);
    resolve.current = undefined;
  };

  // Unmounted while asking: the vault stays locked, as with "Apri senza segreti".
  useEffect(() => () => resolve.current?.(false), []);

  const ensureUnlocked = useCallback(async () => {
    const vault = await queryClient.fetchQuery({ ...vaultStatusQuery, staleTime: 0 });
    if (vault.kind !== "file" || (vault.exists && !vault.locked)) return true;
    // A request still open gives up for the new one.
    resolve.current?.(false);
    return new Promise<boolean>((done) => {
      resolve.current = done;
      setAsking(vault.exists ? "unlock" : "create");
    });
  }, [queryClient]);

  const dialog = (
    <>
      <UnlockVaultDialog
        open={asking === "unlock"}
        onClose={() => finish(false)}
        onUnlocked={() => finish(true)}
      />
      <CreateVaultDialog
        open={asking === "create"}
        onClose={() => finish(false)}
        onCreated={() => finish(true)}
      />
    </>
  );
  return { ensureUnlocked, dialog };
}
