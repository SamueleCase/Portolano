import type { InvokeArgs } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";

import type { IpcError } from "../ipc/result";
import * as deletes from "./fakeBackendDelete";
import * as documents from "./fakeBackendDocument";
import { duplicateNode } from "./fakeBackendDuplicate";
import { editNode } from "./fakeBackendEdit";
import * as misc from "./fakeBackendMisc";
import {
  type FakeBackendState,
  FakeFailure,
  fileName,
  openDocument,
  walk,
} from "./fakeBackendModel";
import * as nodes from "./fakeBackendNodes";
import { sampleDocument } from "./fakeBackendSample";
import * as vault from "./fakeBackendVault";
import { detailsView } from "./fakeBackendDetails";
import { treeView } from "./fakeBackendViews";

export type { FakeBackendState } from "./fakeBackendModel";
export { TEST_MASTER_PASSWORD } from "./fakeBackendModel";

/** The state the tests start from: the sample document open, a locked file vault. */
export function initialFakeState(): FakeBackendState {
  const state: FakeBackendState = {
    defaultDocumentPath:
      "/Users/utente-di-prova/Library/Application Support/dev.portolano.Portolano/portolano.json",
    vault: {
      kind: "file",
      path: "/Users/utente-di-prova/Library/Application Support/dev.portolano.Portolano/vault.json",
      exists: true,
      locked: true,
    },
    document: null,
    files: {},
    recent: [],
    preferences: { density: "normal", background: "on", reduceMotion: false },
    config: { "vault.backend": "file" },
    customIcons: [],
    pendingDeletes: {},
    dialogs: {
      document: null,
      newDocument: null,
      importFile: null,
      icon: null,
      report: null,
      keyFile: null,
      configFiles: {},
    },
    chosen: {},
    clock: () => Date.now(),
    lastImport: false,
    nextId: 0x100,
  };
  const sample = sampleDocument(state, "/Users/utente-di-prova/infra/portolano.json");
  state.files[sample.path] = sample;
  state.document = sample;
  state.recent = [sample.path];
  return state;
}

/**
 * An argument of a command. The generated bindings send each command its
 * arguments with the types of `bindings.ts`, which the dispatcher names
 * through `T`: this cast is that contract, the only one of the fake.
 */
function arg<T>(args: InvokeArgs | undefined, name: string): T {
  const named =
    args !== undefined &&
    !Array.isArray(args) &&
    !ArrayBuffer.isView(args) &&
    !(args instanceof ArrayBuffer)
      ? args
      : {};
  return named[name] as T;
}

/** A string argument of a command. */
function text(args: InvokeArgs | undefined, name: string): string {
  return arg<string>(args, name);
}

/** The answer of every command of the bindings except the sessions, by name. */
function answer(state: FakeBackendState, command: string, args: InvokeArgs | undefined): unknown {
  switch (command) {
    case "app_status":
      return documents.appStatus(state);
    case "choose_document":
      return documents.choose(state, "document");
    case "choose_new_document":
      return documents.choose(state, "newDocument");
    case "open_document":
      return documents.openDocumentAt(state, arg(args, "source"));
    case "create_document":
      return documents.createDocument(state, arg(args, "source"), text(args, "name"));
    case "recent_documents":
      return documents.recentDocuments(state);
    case "reload_document":
      return documents.reloadDocument(state);
    case "close_document":
      return documents.closeDocument(state);
    case "tree":
      return treeView(state, openDocument(state));
    case "node_details":
      return detailsView(state, openDocument(state), text(args, "id"));
    case "search":
      return search(state, text(args, "query"));
    case "add_node":
      return nodes.addNode(state, arg(args, "node"));
    case "edit_node":
      return editNode(state, text(args, "id"), arg(args, "edit"));
    case "move_node":
      return nodes.moveNode(
        state,
        text(args, "id"),
        arg(args, "destination"),
        arg(args, "placement"),
      );
    case "choose_key_file":
      return state.dialogs.keyFile;
    case "duplicate_node":
      return duplicateNode(state, text(args, "id"));
    case "preview_delete":
      return deletes.previewDelete(state, text(args, "id"));
    case "delete_node":
      return deletes.deleteNode(state, text(args, "id"), arg(args, "options"));
    case "undo_delete":
      return deletes.undoDelete(state, text(args, "undoId"));
    case "secret_status":
      return vault.secretStatus(state, text(args, "id"));
    case "set_secret":
      return vault.setSecret(state, text(args, "id"), arg(args, "slot"));
    case "clear_secret":
      return vault.clearSecret(state, text(args, "id"), arg(args, "slot"));
    case "vault_status":
      return vault.vaultStatus(state);
    case "unlock_vault":
      return vault.unlockVault(state, text(args, "masterPassword"));
    case "lock_vault":
      return vault.lockVault(state);
    case "create_file_vault":
      return vault.createFileVault(state, text(args, "masterPassword"));
    case "validate":
      return misc.validate(state);
    case "choose_import_file":
      return documents.choose(state, "importFile");
    case "import_preview":
      return misc.importPreview(state, arg(args, "options"));
    case "import_apply":
      return misc.importApply(state, arg(args, "options"), arg(args, "answers"));
    case "save_import_report":
      return misc.saveImportReport(state);
    case "connect_preview":
      return misc.connectPreview(state, text(args, "id"));
    case "login_request":
      return misc.loginRequest(state, text(args, "id"));
    case "config_get":
      return documents.configEntries(state);
    case "config_set":
      return documents.configSet(state, text(args, "key"), text(args, "value"));
    case "config_reset":
      return documents.configReset(state, text(args, "key"));
    case "choose_config_file":
      return documents.chooseConfigFile(state, text(args, "key"));
    case "ui_prefs_get":
      return { ...state.preferences };
    case "ui_prefs_set":
      return documents.uiPrefsSet(state, arg(args, "preferences"));
    case "builtin_icons":
      return documents.builtinIcons();
    case "custom_icons":
      return documents.customIcons(state);
    case "choose_icon_file":
      return documents.choose(state, "icon");
    case "add_icon":
      return documents.addIcon(state);
    default:
      throw new FakeFailure("internal", `comando non simulato: ${command}`);
  }
}

/** The nodes whose name, host or tag contains `query`, ignoring case. */
function search(state: FakeBackendState, query: string): string[] {
  const wanted = query.trim().toLowerCase();
  if (wanted === "") return [];
  return [...walk(openDocument(state).nodes)]
    .filter(({ node }) =>
      [node.name, node.host ?? "", ...node.tags].some((value) =>
        value.toLowerCase().includes(wanted),
      ),
    )
    .map(({ node }) => node.id);
}

/** The commands after which the app saved the document and sends `document-changed`. */
const SAVING = new Set([
  "add_node",
  "edit_node",
  "move_node",
  "duplicate_node",
  "delete_node",
  "undo_delete",
  "set_secret",
  "clear_secret",
  "import_apply",
  "reload_document",
]);

/** The events the backend sends after `command` succeeded. */
function eventsAfter(state: FakeBackendState, command: string): void {
  if (SAVING.has(command) && state.document) {
    void emit("document-changed", { fileName: fileName(state.document.path) });
  }
  if (command === "lock_vault") {
    void emit("vault-locked", { kind: state.vault.kind });
  }
}

/**
 * Answers the commands of the generated bindings from `state`, in memory, in
 * place of the IPC of Tauri, with the semantics of the Rust commands (errors
 * included): for the component tests and the end-to-end tests.
 *
 * Events reach the listeners of the page (`emit` of `@tauri-apps/api/event`):
 * `document-changed` after every save and reload, `vault-locked` after
 * `lock_vault`. Deletions expire by `state.clock`. Not simulated: the
 * watcher of the file (`document-modified-externally`, `document-missing`:
 * a test emits them itself) and the menu (`menu-action`, likewise).
 */
export function installFakeBackend(state: FakeBackendState = initialFakeState()): FakeBackendState {
  mockIPC(
    (command, args) => {
      try {
        deletes.expireDeletions(state);
        const result = answer(state, command, args);
        eventsAfter(state, command);
        return result;
      } catch (failure) {
        if (failure instanceof FakeFailure) {
          return rejectWith(failure.error);
        }
        throw failure;
      }
    },
    { shouldMockEvents: true },
  );
  return state;
}

/** A failed command: the backend rejects with the error value itself. */
export function rejectWith(error: IpcError): Promise<never> {
  // Commands reject with an `IpcError` object, not an `Error`: the generated
  // bindings tell the two apart to return command errors as values.
  // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
  return Promise.reject(error);
}
