import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import type { SecretSlotInput } from "../../bindings";
import { presentFailure } from "../../ipc/errors";
import { useSetSecret } from "../../ipc/mutations";
import { vaultStatusQuery } from "../../ipc/queries";
import { CreateVaultDialog } from "./CreateVaultDialog";
import { readablePath } from "./nodePaths";
import { SecretDialog } from "./SecretDialog";
import { UnlockVaultDialog } from "./UnlockVaultDialog";
import { vaultBackend } from "./vault";

/** A secret to set at once, outside the editor ("Imposta…" of the detail). */
export interface SecretRequest {
  nodeId: string;
  /** The path of the node, as the backend writes it. */
  nodePath: string;
  slot: SecretSlotInput;
  /** The secret is set already: "Sostituisci". */
  replacing: boolean;
}

interface SetSecretDialogProps {
  /** The secret to ask for; `null` closes the dialog. */
  request: SecretRequest | null;
  onClose: () => void;
  /** The secret is in the vault. */
  onSaved?: () => void;
}

/**
 * The password dialog that saves at once with `set_secret` (the
 * `requestPassword` of the frame, DEC-035): with the file vault locked or
 * missing it asks for the master password first, so that the secret is
 * typed only when it can be saved.
 */
export function SetSecretDialog({ request, onClose, onSaved }: SetSecretDialogProps) {
  const { data: vault } = useQuery({ ...vaultStatusQuery, enabled: request !== null });
  const setSecret = useSetSecret();
  const [error, setError] = useState<string>();
  if (request === null || vault === undefined) return null;

  const close = () => {
    setError(undefined);
    onClose();
  };
  // Once the vault opens its query refreshes, and the secret dialog follows.
  if (vault.kind === "file" && !vault.exists) {
    return <CreateVaultDialog open onClose={close} onCreated={() => undefined} />;
  }
  if (vault.kind === "file" && vault.locked) {
    return <UnlockVaultDialog open onClose={close} onUnlocked={() => undefined} />;
  }
  return (
    <SecretDialog
      open
      slot={request.slot}
      nodeLabel={readablePath(request.nodePath)}
      replacing={request.replacing}
      mode="save"
      backend={vaultBackend(vault.kind)}
      busy={setSecret.isPending}
      error={error}
      onClose={close}
      onConfirm={(value) => {
        setError(undefined);
        setSecret
          .mutateAsync({ id: request.nodeId, slot: request.slot, value })
          .then(() => {
            onClose();
            onSaved?.();
          })
          .catch((failure: unknown) => setError(presentFailure(failure).message));
      }}
    />
  );
}
