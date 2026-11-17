import type { LoginProps, PasswordProps, VaultDialogProps } from "../../app/features";
import { CreateVaultDialog } from "./CreateVaultDialog";
import { LoginDialog } from "./LoginDialog";
import { SetSecretDialog } from "./SetSecretDialog";
import { UnlockVaultDialog } from "./UnlockVaultDialog";

/**
 * The dialogs of UI-WP5 at the hook points of the window (`app/features.ts`):
 * each one opens on its request and answers it.
 */

/** `password`: "Imposta…" / "Sostituisci…" of a secret of the detail. */
export function PasswordFeature({ node, field, onClose }: PasswordProps) {
  return <SetSecretDialog node={node} field={field} onClose={onClose} />;
}

/** `unlockVault`: the master password of the file vault. */
export function UnlockVaultFeature({ onDone }: VaultDialogProps) {
  return <UnlockVaultDialog open onClose={() => onDone(false)} onUnlocked={() => onDone(true)} />;
}

/** `createFileVault`: a new file vault with its master password. */
export function CreateVaultFeature({ onDone }: VaultDialogProps) {
  return <CreateVaultDialog open onClose={() => onDone(false)} onCreated={() => onDone(true)} />;
}

/** `login`: the credentials asked at launch, handed to the caller only. */
export function LoginFeature({ request, onDone }: LoginProps) {
  return <LoginDialog request={request} onLogin={onDone} onClose={() => onDone(null)} />;
}
