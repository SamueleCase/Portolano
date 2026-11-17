import {
  CreateVaultFeature,
  LoginFeature,
  PasswordFeature,
  UnlockVaultFeature,
} from "../features/dialogs/DialogFeatures";
import { EditorFeature } from "../features/editor/EditorFeature";
import { ImportWizard } from "../features/import/ImportWizard";
import { PreferencesFeature } from "../features/preferences/PreferencesFeature";
import { ValidationPanel } from "../features/validation/ValidationPanel";
import type { FeatureRegistry } from "./features";

/**
 * The functions mounted at the hook points of the window (`features.ts`).
 * UI-WP5 (editor, password, vault, login, preferences) and UI-WP6 (import,
 * validation) add their line here when they integrate (DEC-035).
 */
export const featureRegistry: FeatureRegistry = {
  editor: EditorFeature,
  password: PasswordFeature,
  unlockVault: UnlockVaultFeature,
  createFileVault: CreateVaultFeature,
  login: LoginFeature,
  preferences: PreferencesFeature,
  import: ImportWizard,
  validation: ValidationPanel,
};
