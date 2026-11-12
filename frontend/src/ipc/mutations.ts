import {
  commands,
  type DeleteOptionsInput_Deserialize,
  type DocumentSource,
  type ImportAnswerInput_Deserialize,
  type ImportOptionsInput_Deserialize,
  type InterfacePreferences,
  type NewNodeInput_Deserialize,
  type NodeEditInput_Deserialize,
  type PlacementInput,
  type SecretSlotInput,
} from "../bindings";
import { queryKeys } from "./keys";
import { useCommandMutation, useSecretMutation } from "./mutation";

/**
 * The commands that change something, as TanStack Query mutations that make
 * the data they change stale. The backend also sends `document-changed`
 * after every save (`events.ts`), which covers the changes made by menus
 * and other windows.
 *
 * The mutations whose arguments can carry a secret (a node with its
 * passwords, a password, a master password, the answers of an import) use
 * `useSecretMutation`: the secret never becomes a variable of Query.
 */

/** What a change of document makes stale. */
const documentSwitch = [
  queryKeys.appStatus,
  queryKeys.recentDocuments,
  queryKeys.document,
  queryKeys.vault,
] as const;

export const useOpenDocument = () =>
  useCommandMutation((source: DocumentSource) => commands.openDocument(source), documentSwitch);

export const useCreateDocument = () =>
  useCommandMutation(
    ({ source, name }: { source: DocumentSource; name: string }) =>
      commands.createDocument(source, name),
    documentSwitch,
  );

export const useReloadDocument = () =>
  useCommandMutation(() => commands.reloadDocument(), [queryKeys.document]);

export const useCloseDocument = () =>
  useCommandMutation(() => commands.closeDocument(), documentSwitch);

export const useChooseDocument = () => useCommandMutation(() => commands.chooseDocument(), []);

export const useChooseNewDocument = () =>
  useCommandMutation(() => commands.chooseNewDocument(), []);

export const useAddNode = () =>
  useSecretMutation(
    (node: NewNodeInput_Deserialize) => commands.addNode(node),
    [queryKeys.document],
  );

export const useEditNode = () =>
  useSecretMutation(
    ({ id, edit }: { id: string; edit: NodeEditInput_Deserialize }) => commands.editNode(id, edit),
    [queryKeys.document],
  );

export const useMoveNode = () =>
  useCommandMutation(
    ({
      id,
      destination,
      placement,
    }: {
      id: string;
      destination: string | null;
      placement: PlacementInput;
    }) => commands.moveNode(id, destination, placement),
    [queryKeys.document],
  );

export const useDuplicateNode = () =>
  useCommandMutation((id: string) => commands.duplicateNode(id), [queryKeys.document]);

/** What deleting a node would remove: read on demand, changes nothing. */
export const usePreviewDelete = () =>
  useCommandMutation((id: string) => commands.previewDelete(id), []);

export const useDeleteNode = () =>
  useCommandMutation(
    ({ id, options }: { id: string; options: DeleteOptionsInput_Deserialize }) =>
      commands.deleteNode(id, options),
    [queryKeys.document],
  );

export const useUndoDelete = () =>
  useCommandMutation((undoId: string) => commands.undoDelete(undoId), [queryKeys.document]);

export const useSetSecret = () =>
  useSecretMutation(
    ({ id, slot, value }: { id: string; slot: SecretSlotInput; value: string }) =>
      commands.setSecret(id, slot, value),
    [queryKeys.document],
  );

export const useClearSecret = () =>
  useCommandMutation(
    ({ id, slot }: { id: string; slot: SecretSlotInput }) => commands.clearSecret(id, slot),
    [queryKeys.document],
  );

/** What a change of the vault makes stale: its state and every secret state. */
const vaultChange = [queryKeys.vault, queryKeys.document] as const;

export const useUnlockVault = () =>
  useSecretMutation((masterPassword: string) => commands.unlockVault(masterPassword), vaultChange);

export const useCreateFileVault = () =>
  useSecretMutation(
    (masterPassword: string) => commands.createFileVault(masterPassword),
    vaultChange,
  );

export const useLockVault = () => useCommandMutation(() => commands.lockVault(), vaultChange);

export const useChooseImportFile = () => useCommandMutation(() => commands.chooseImportFile(), []);

/** What importing would do, and the passwords to ask: changes nothing. */
export const useImportPreview = () =>
  useCommandMutation(
    (options: ImportOptionsInput_Deserialize) => commands.importPreview(options),
    [],
  );

export const useImportApply = () =>
  useSecretMutation(
    ({
      options,
      answers,
    }: {
      options: ImportOptionsInput_Deserialize;
      answers: ImportAnswerInput_Deserialize[];
    }) => commands.importApply(options, answers),
    [queryKeys.document],
  );

export const useSaveImportReport = () => useCommandMutation(() => commands.saveImportReport(), []);

/** What a change of the configuration makes stale: the vault may change too. */
const configChange = [
  queryKeys.config,
  queryKeys.appStatus,
  queryKeys.vault,
  queryKeys.document,
] as const;

export const useConfigSet = () =>
  useCommandMutation(
    ({ key, value }: { key: string; value: string }) => commands.configSet(key, value),
    configChange,
  );

export const useConfigReset = () =>
  useCommandMutation((key: string) => commands.configReset(key), configChange);

export const useChooseConfigFile = () =>
  useCommandMutation((key: string) => commands.chooseConfigFile(key), configChange);

export const useUiPrefsSet = () =>
  useCommandMutation(
    (preferences: InterfacePreferences) => commands.uiPrefsSet(preferences),
    [queryKeys.uiPreferences, queryKeys.appStatus],
  );

export const useChooseIconFile = () => useCommandMutation(() => commands.chooseIconFile(), []);

export const useAddIcon = () =>
  useCommandMutation(() => commands.addIcon(), [queryKeys.customIcons]);
