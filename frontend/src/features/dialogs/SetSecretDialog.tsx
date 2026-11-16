import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import type { NodeRef, SecretSlotInput, SecretView } from "../../bindings";
import { presentFailure } from "../../ipc/errors";
import { useSetSecret } from "../../ipc/mutations";
import { nodeDetailsQuery } from "../../ipc/queries";
import { requestVault } from "../../state/requests";
import { readablePath } from "./nodePaths";
import { SecretDialog } from "./SecretDialog";
import { useVaultState } from "./vault";

interface SetSecretDialogProps {
  /** The node of the secret, as the detail names it. */
  node: NodeRef;
  /** The secret to ask for. */
  field: SecretSlotInput;
  /** Closes the dialog: the secret is in the vault, or the user gave up. */
  onClose: () => void;
}

/** Whether `secrets` hold the one of `field`: then the dialog says "Sostituisci". */
function holds(secrets: readonly SecretView[], field: SecretSlotInput): boolean {
  return secrets.some(({ slot }) => {
    switch (field.kind) {
      case "password":
      case "passphrase":
        return slot === field.kind;
      case "privateKey":
        return slot === "private_key";
      case "property":
        return typeof slot === "object" && slot.property === field.name;
    }
  });
}

/**
 * The password dialog that saves at once with `set_secret`: "Imposta…" and
 * "Sostituisci…" of the detail (the hook point `password`). The file vault
 * is made ready first by the window (`requestVault`, DEC-041.1), so that the
 * secret is typed only when it can be saved; without it the dialog closes.
 */
export function SetSecretDialog({ node, field, onClose }: SetSecretDialogProps) {
  const { backend } = useVaultState();
  const { data: details } = useQuery(nodeDetailsQuery(node.id));
  const setSecret = useSetSecret();
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string>();

  // The latest `onClose`, for the answer of the vault that comes later.
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  });

  // Once per dialog: the vault is asked when it opens.
  useEffect(() => {
    let mounted = true;
    requestVault()
      .then((open) => {
        if (mounted && open) setReady(true);
        else if (mounted) close.current();
      })
      .catch((failure: unknown) => {
        // The state of the vault could not be read: the dialog says why.
        if (!mounted) return;
        setError(presentFailure(failure).message);
        setReady(true);
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <SecretDialog
      open={ready}
      slot={field}
      nodeLabel={readablePath(node.path)}
      replacing={holds(details?.secrets ?? [], field)}
      mode="save"
      backend={backend}
      busy={setSecret.isPending}
      error={error}
      onClose={onClose}
      onConfirm={(value) => {
        setError(undefined);
        setSecret
          .mutateAsync({ id: node.id, slot: field, value })
          .then(onClose)
          .catch((failure: unknown) => setError(presentFailure(failure).message));
      }}
    />
  );
}
