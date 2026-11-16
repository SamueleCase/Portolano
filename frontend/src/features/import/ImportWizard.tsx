import { useRef } from "react";

import { Dialog } from "../../components/Dialog";
import { Stepper } from "../../components/Stepper";
import { texts } from "../../i18n/it";
import { FileStep } from "./FileStep";
import styles from "./ImportWizard.module.css";
import { OptionsStep } from "./OptionsStep";
import { PasswordStep } from "./PasswordStep";
import { ReportStep } from "./ReportStep";
import { useDestination } from "./useDestination";
import { useImportWizard } from "./useImportWizard";
import { useReportActions } from "./useReportActions";
import { ReportActions, WizardFooter } from "./WizardActions";
import { wizardSteps } from "./wizardSteps";

/** The props of the import wizard, the API its host uses (SPEC-UI §5.4). */
export interface ImportWizardProps {
  open: boolean;
  /** Esc, "Annulla", "Vai all'albero": the host closes the wizard. */
  onClose: () => void;
  /**
   * A row of the report or "Vai all'albero": the host selects the node and
   * shows it. The wizard stays open, except after "Vai all'albero".
   */
  onGoToNode: (id: string) => void;
  /** "Sblocca il vault…" of step 2, when the passwords cannot be written yet. */
  onUnlockVault?: (() => void) | undefined;
}

/**
 * The import wizard of Royal TSX documents (inventario §5.13), four steps in
 * a dialog. Closed, it keeps nothing: the next opening starts from the file.
 */
export function ImportWizard({ open, ...props }: ImportWizardProps) {
  return open ? <OpenImportWizard {...props} /> : null;
}

function OpenImportWizard({ onClose, onGoToNode, onUnlockVault }: Omit<ImportWizardProps, "open">) {
  const wizard = useImportWizard();
  const report = useReportActions(wizard.outcome?.result ?? null, onGoToNode);
  const destination = useDestination(wizard.settings.into, (into) =>
    wizard.changeSettings({ into }),
  );
  const passwordField = useRef<HTMLInputElement>(null);
  const { step } = wizard;
  const passwordsSkipped =
    !wizard.settings.askPasswords || (wizard.preview !== null && wizard.requests.length === 0);

  /** The typed password moves to the answers and the field empties at once. */
  const savePassword = () => {
    const field = passwordField.current;
    const value = field?.value ?? "";
    if (field) field.value = "";
    wizard.answer(value);
  };
  const skipPassword = () => {
    if (passwordField.current) passwordField.current.value = "";
    wizard.answer(null);
  };

  const request = wizard.requests[wizard.progress.position];
  const canLeaveOptions =
    wizard.preview !== null && !wizard.previewing && !(wizard.asking && wizard.vaultLocked);
  const footer =
    step === "file" ? (
      <WizardFooter
        step="file"
        applying={wizard.applying}
        canGoOn={wizard.reading?.state === "readable"}
        onCancel={onClose}
      />
    ) : step === "options" ? (
      <WizardFooter
        step="options"
        applying={wizard.applying}
        canGoOn={canLeaveOptions}
        onBack={wizard.back}
      />
    ) : step === "passwords" ? (
      <WizardFooter
        step="passwords"
        applying={wizard.applying}
        onSkip={skipPassword}
        onSkipAll={wizard.skipRemaining}
      />
    ) : undefined;

  return (
    <>
      <Dialog
        open
        onClose={onClose}
        title={texts.importWizard.title}
        subtitle={wizard.file?.fileName}
        width={step === "report" ? "report" : "wizard"}
        footer={footer}
        onSubmit={step === "passwords" ? savePassword : step === "report" ? undefined : wizard.next}
      >
        <Stepper steps={wizardSteps(step, passwordsSkipped)} />
        {wizard.applying ? (
          <p role="status" className={styles.muted}>
            {texts.importWizard.options.applying}
          </p>
        ) : (
          <>
            {step === "file" && (
              <FileStep
                fileName={wizard.file?.fileName ?? null}
                reading={wizard.reading}
                onChoose={() => void wizard.chooseFile()}
                choosing={wizard.choosing}
              />
            )}
            {step === "options" && (
              <OptionsStep
                settings={wizard.settings}
                onChange={wizard.changeSettings}
                destination={destination.label}
                importedFolder={wizard.documentName}
                onChooseDestination={destination.choose}
                preview={wizard.preview}
                previewing={wizard.previewing}
                error={wizard.error}
                passwordsDropped={wizard.passwordsDropped}
                trashSize={wizard.trashSize}
                vaultLocked={wizard.vaultLocked}
                onUnlockVault={onUnlockVault}
              />
            )}
            {step === "passwords" && request && (
              <PasswordStep
                key={wizard.progress.position}
                request={request}
                position={wizard.progress.position}
                total={wizard.requests.length}
                entered={wizard.progress.entered}
                skipped={wizard.progress.skipped}
                fieldRef={passwordField}
              />
            )}
            {step === "report" && wizard.outcome && (
              <ReportStep
                result={wizard.outcome.result}
                passwordsAsked={wizard.outcome.passwordsAsked}
                onGoToItem={(item) => void report.goToItem(item)}
                notice={report.notice}
                actions={
                  <ReportActions
                    onExport={() => void report.exportJson()}
                    exporting={report.exporting}
                    onGoToTree={() => {
                      if (wizard.outcome) onGoToNode(wizard.outcome.result.root);
                      onClose();
                    }}
                  />
                }
              />
            )}
          </>
        )}
      </Dialog>
      {/* Outside the wizard in the React tree too: its keys must not reach the wizard. */}
      {destination.picker}
    </>
  );
}
