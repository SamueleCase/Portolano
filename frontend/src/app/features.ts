/**
 * The hook points of the main window (DEC-035, DEC-041.1): where the editor
 * and the dialogs (UI-WP5) and the import and the validation (UI-WP6) mount.
 *
 * ## The requests of the user
 * The toolbar, the context menu, the detail, the native menu (`menu-action`)
 * and the shortcuts ask for these through the actions of the window
 * (`useWindowActions()`, `windowActions.ts`), which the mounted functions can
 * use too:
 * - `create(kind)` / `edit(id)`: the editor on `{ mode: "add", kind: "ssh" |
 *   "rdp" | "folder" | "credential", parent }` (the parent is the selected
 *   folder, the folder of the selected node, or `null` for the root:
 *   ambiguity 10) or `{ mode: "edit", id }`; → registration `editor`;
 * - `requestPassword(node, field)`: "Imposta…" / "Sostituisci…" of a secret
 *   of the detail, written at once with `set_secret`; → `password`;
 * - `unlockVault()`, `createFileVault()`: the footer of the vault; →
 *   `unlockVault`, `createFileVault`;
 * - `openImport()`: "Importa", ⌘I, the empty document; → `import`;
 * - `openValidation()`: "Valida", ⇧⌘V; opens the tab "Validazione", whose body
 *   is the registration `validation`;
 * - `openPreferences()`: "Preferenze", ⌘,; → `preferences`.
 * A request of the user opens only when nothing else is open: with a dialog
 * or the editor open it is refused, not put in place of it.
 *
 * ## The requests of the code (`state/requests.ts`)
 * A function already open asks for another one on top of itself, and stays
 * mounted below with its state (a stack):
 * - `requestVault() → Promise<boolean>`: the file vault ready for secrets
 *   (the editor before saving them, the import before writing them): nothing
 *   if it is open, the creation (`createFileVault`) if its file does not
 *   exist yet, else the unlock (`unlockVault`); resolves to whether it is now
 *   open, `false` also when that dialog is not mounted. The window does the
 *   same when a command fails with `vaultLocked`, and repeats the command
 *   once the vault is open (DEC-041.2);
 * - `requestLogin(request) → Promise<SessionLogin | null>`: the login of a
 *   credential "ask at launch" (DEC-016.3), before a connection and before
 *   "Riprova" (DEC-033.8); the login goes from `onDone` to the caller and is
 *   never kept (SPEC-UI §1.2);
 * - `pushRequest(request)`: any other request, on top.
 *
 * ## The registrations
 * A function declares its component in `featureRegistry` (`featureRegistry.ts`,
 * the integration lines of UI-WP5 and UI-WP6). `FeatureHost` mounts the
 * component of every open request, the last on top, inside the providers of
 * the window (queries, sessions, actions). A component closes with `onClose`;
 * those that answer (login, vault) with `onDone`, which also closes them and
 * answers their own request even when another is on top. A request whose
 * function is not mounted closes at once with "no" (`null`, `false`). A
 * request without a registration is not offered: its buttons and menu
 * entries are disabled with the reason, and a request that still arrives (a
 * native menu) is answered with that reason in a toast.
 *
 * After a change the components invalidate the TanStack Query data with the
 * mutations of `ipc/`; the window follows (`document-changed` too).
 */
import { type ComponentType, createContext, useContext } from "react";

import type { LoginRequest, NodeRef, SecretSlotInput } from "../bindings";
import type { SessionLogin } from "../ipc/sessions";
import type { EditorTarget } from "../state/requests";

export type { EditorTarget, NewNodeKind } from "../state/requests";

/** What every dialog or panel of a hook point receives. */
export interface ClosableProps {
  /** Closes it; the request is over. */
  onClose: () => void;
}

/** The editor in two steps (inventario §5.5). */
export interface EditorProps extends ClosableProps {
  target: EditorTarget;
  /** The node was saved: the window selects it (a new node, a renamed one). */
  onSaved: (id: string) => void;
}

/** The password dialog of a secret of the detail (inventario §5.9). */
export interface PasswordProps extends ClosableProps {
  node: NodeRef;
  field: SecretSlotInput;
}

/** The login asked at launch (DEC-016.3). */
export interface LoginProps {
  request: LoginRequest;
  /** The login typed, or `null` for "Annulla"; the dialog keeps nothing. */
  onDone: (login: SessionLogin | null) => void;
}

/** The unlock and the creation of the file vault (inventario §5.10). */
export interface VaultDialogProps {
  /** Whether the vault is now open (`false` for "Annulla"); closes the dialog. */
  onDone: (done: boolean) => void;
}

/** The import wizard (inventario §5.13). */
export interface ImportProps extends ClosableProps {
  /** "Vai all'albero" or a row of the report: the window selects the node. */
  onGoToNode: (id: string) => void;
}

/** The body of the validation tab (inventario §5.14). */
export interface ValidationProps {
  /** "Vai al nodo →": the window selects the node and shows its detail. */
  onGoToNode: (id: string) => void;
}

/** The components of the hook points; an absent one is not offered. */
export interface FeatureRegistry {
  editor?: ComponentType<EditorProps>;
  password?: ComponentType<PasswordProps>;
  unlockVault?: ComponentType<VaultDialogProps>;
  createFileVault?: ComponentType<VaultDialogProps>;
  login?: ComponentType<LoginProps>;
  import?: ComponentType<ImportProps>;
  preferences?: ComponentType<ClosableProps>;
  validation?: ComponentType<ValidationProps>;
}

/** A hook point. */
export type Feature = keyof FeatureRegistry;

/** The registrations the window uses; the tests give their own. */
export const FeatureRegistryContext = createContext<FeatureRegistry>({});

/** The registrations of the window. */
export function useFeatures(): FeatureRegistry {
  return useContext(FeatureRegistryContext);
}
