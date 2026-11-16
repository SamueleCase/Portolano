/**
 * The hook points of the main window (DEC-035): where the editor and the
 * dialogs (UI-WP5) and the import and the validation (UI-WP6) mount.
 *
 * ## The requests
 * The toolbar, the context menu, the detail, the native menu (`menu-action`)
 * and the shortcuts ask for these, with the functions of `state/requests.ts`
 * and `state/workspace.ts`:
 * - `openEditor(target)`: `{ mode: "add", kind: "ssh" | "rdp" | "folder" |
 *   "credential", parent }` (the parent is the selected folder, the folder of
 *   the selected node, or `null` for the root: ambiguity 10) or `{ mode:
 *   "edit", id }`; → registration `editor`;
 * - `requestPassword(node, field)`: "Imposta…" / "Sostituisci…" of a secret
 *   of the detail, written at once with `set_secret`; → `password`;
 * - `unlockVault()`, `createFileVault()`: the footer of the vault and the
 *   errors `vaultLocked`; → `unlockVault`, `createFileVault`;
 * - `requestLogin(request) → Promise<SessionLogin | null>`: before a
 *   connection whose `login_request` is not `null`, and before "Riprova" of
 *   such a session (DEC-033.8); → `login`. The login goes from `onDone` to
 *   the connection and is never kept (SPEC-UI §1.2);
 * - `openImport()`: "Importa", ⌘I, the empty document; → `import`;
 * - `openValidation()`: "Valida", ⇧⌘V; opens the tab "Validazione", whose body
 *   is the registration `validation`;
 * - `openPreferences()`: "Preferenze", ⌘,; → `preferences`.
 *
 * ## The registrations
 * A function declares its component in `featureRegistry` (`featureRegistry.ts`,
 * the integration lines of UI-WP5 and UI-WP6). The window mounts it while its
 * request is open, one at a time (`FeatureHost`); the component closes with
 * `onClose`. A request without a registration is not offered: its buttons and
 * menu entries are disabled, with the reason in the tooltip, and a request that
 * still arrives (a native menu) is answered with that reason in a toast.
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
  unlockVault?: ComponentType<ClosableProps>;
  createFileVault?: ComponentType<ClosableProps>;
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
