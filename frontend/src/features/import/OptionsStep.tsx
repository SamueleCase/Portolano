import { useId } from "react";

import type { ConflictChoice, ImportPreviewView_Serialize } from "../../bindings";
import { Button } from "../../components/Button";
import { FormField } from "../../components/FormField";
import { SegmentedControl } from "../../components/SegmentedControl";
import { ToggleRow } from "../../components/ToggleRow";
import { visibleText } from "../../components/visibleText";
import { interpolate, type PluralForms, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./ImportWizard.module.css";

/** The choices of step 2 (SPEC §8.4). */
export interface ImportSettings {
  /** The destination folder; none is the root of the document. */
  into: string | null;
  onConflict: ConflictChoice;
  includeTrash: boolean;
  /** Step 3: off, every password is skipped. */
  askPasswords: boolean;
}

interface OptionsStepProps {
  settings: ImportSettings;
  onChange: (change: Partial<ImportSettings>) => void;
  /** The readable path of the destination, "Radice del documento" for the root. */
  destination: string;
  /** The folder the import creates there: the name of the imported document. */
  importedFolder: string | null;
  onChooseDestination: () => void;
  /** The dry run of the current settings, none while it runs or when it failed. */
  preview: ImportPreviewView_Serialize | null;
  previewing: boolean;
  /** Why the dry run or the import failed, in the words of the backend. */
  error: string | null;
  /** A failed import dropped the passwords typed for it: they have to be typed again. */
  passwordsDropped: boolean;
  /** The items in the trash of the file, skipped unless included. */
  trashSize: number;
  /** The passwords cannot be written until the file vault is unlocked. */
  vaultLocked: boolean;
  onUnlockVault?: (() => void) | undefined;
}

const CONFLICTS: readonly ConflictChoice[] = ["fail", "update", "newIds"];

function countText(count: number, forms: PluralForms & { zero: string }): string {
  return count === 0 ? forms.zero : plural(count, forms);
}

/**
 * Step 2 of the wizard (inventario §5.13): destination, conflicts, trash and
 * passwords, with the counts of the dry run of `import_preview`.
 */
export function OptionsStep({
  settings,
  onChange,
  destination,
  importedFolder,
  onChooseDestination,
  preview,
  previewing,
  error,
  passwordsDropped,
  trashSize,
  vaultLocked,
  onUnlockVault,
}: OptionsStepProps) {
  const destinationId = useId();
  const { options, actions } = texts.importWizard;
  const requests = preview?.requests.length ?? 0;
  return (
    <div className={styles.step}>
      <FormField label={options.destination} controlId={destinationId}>
        <div className={styles.destination}>
          <output id={destinationId} className={styles.destinationPath}>
            {visibleText(
              importedFolder === null
                ? destination
                : `${destination}${texts.validation.separator}${importedFolder}`,
            )}
          </output>
          <Button variant="small" onClick={onChooseDestination}>
            {options.chooseDestination}
          </Button>
        </div>
      </FormField>
      <div className={styles.conflicts}>
        {/* The group is named by its own label: this is the same text, for the eye. */}
        <span className={styles.groupLabel} aria-hidden="true">
          {options.conflicts}
        </span>
        <SegmentedControl
          label={options.conflicts}
          value={settings.onConflict}
          onChange={(onConflict) => onChange({ onConflict })}
          options={CONFLICTS.map((value) => ({ value, label: options.conflictChoices[value] }))}
        />
        <p className={styles.muted}>{options.conflictHints[settings.onConflict]}</p>
      </div>
      <ToggleRow
        label={options.includeTrash}
        description={countText(trashSize, options.trash)}
        checked={settings.includeTrash}
        onChange={(includeTrash) => onChange({ includeTrash })}
      />
      <ToggleRow
        label={options.askPasswords}
        description={countText(requests, options.passwords)}
        checked={settings.askPasswords}
        onChange={(askPasswords) => onChange({ askPasswords })}
      />
      <p role="status" className={styles.muted}>
        {previewing
          ? options.previewing
          : preview &&
            interpolate(options.preview, {
              inserted: plural(preview.result.inserted, options.inserted),
              updated: plural(preview.result.updated, options.updated),
            })}
      </p>
      {vaultLocked && settings.askPasswords && requests > 0 && (
        <div className={styles.vaultLocked}>
          <p className={styles.muted}>{options.vaultLocked}</p>
          {onUnlockVault && (
            <Button variant="small" onClick={onUnlockVault}>
              {actions.unlockVault}
            </Button>
          )}
        </div>
      )}
      {error && (
        <div role="alert" className={styles.failure}>
          <p className={styles.error}>{visibleText(error)}</p>
          {passwordsDropped && <p className={styles.error}>{options.passwordsDropped}</p>}
        </div>
      )}
    </div>
  );
}
