import { type ReactNode, useRef } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { type Step, Stepper } from "../../components/Stepper";
import { texts } from "../../i18n/it";
import { FileStep } from "./FileStep";
import styles from "./ImportWizard.module.css";
import { OptionsStep } from "./OptionsStep";
import { PasswordStep } from "./PasswordStep";
import { ReportStep } from "./ReportStep";
import { useDestination } from "./useDestination";
import { useImportWizard, type WizardStep } from "./useImportWizard";
import { useReportActions } from "./useReportActions";

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

const STEPS: readonly WizardStep[] = ["file", "options", "passwords", "report"];

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
  const { actions, steps } = texts.importWizard;
  const current = STEPS.indexOf(wizard.step);

  const stepper: Step[] = STEPS.map((step, index) => ({
    label: steps[step],
    state:
      step === "passwords" && !wizard.settings.askPasswords
        ? "skipped"
        : index < current
          ? "done"
          : index === current
            ? "current"
            : "upcoming",
  }));

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

  const footers: Record<WizardStep, ReactNode> = {
    file: (
      <>
        <Button onClick={onClose}>{actions.cancel}</Button>
        <Button type="submit" variant="primary" disabled={wizard.reading?.state !== "readable"}>
          {actions.next}
        </Button>
      </>
    ),
    options: (
      <>
        <Button className={styles.footerStart} onClick={wizard.back}>
          {actions.back}
        </Button>
        <Button
          type="submit"
          variant="primary"
          disabled={!canLeaveOptions}
          loading={wizard.applying}
        >
          {actions.next}
        </Button>
      </>
    ),
    passwords: (
      <>
        <Button onClick={skipPassword}>{actions.skip}</Button>
        <Button variant="ghost" className={styles.footerStart} onClick={wizard.skipRemaining}>
          {actions.skipAll}
        </Button>
        <Button type="submit" variant="primary" loading={wizard.applying}>
          {actions.saveAndContinue}
        </Button>
      </>
    ),
    report: (
      <>
        <Button onClick={() => void report.exportJson()} loading={report.exporting}>
          {actions.exportJson}
        </Button>
        <Button
          variant="primary"
          onClick={() => {
            if (wizard.outcome) onGoToNode(wizard.outcome.result.root);
            onClose();
          }}
        >
          {actions.goToTree}
        </Button>
      </>
    ),
  };

  return (
    <>
      <Dialog
        open
        onClose={onClose}
        title={texts.importWizard.title}
        subtitle={wizard.file?.fileName}
        width={wizard.step === "report" ? "report" : "wizard"}
        footer={footers[wizard.step]}
        onSubmit={
          wizard.step === "passwords"
            ? savePassword
            : wizard.step === "report"
              ? undefined
              : wizard.next
        }
      >
        <Stepper steps={stepper} />
        {wizard.step === "file" && (
          <FileStep
            fileName={wizard.file?.fileName ?? null}
            reading={wizard.reading}
            onChoose={() => void wizard.chooseFile()}
            choosing={wizard.choosing}
          />
        )}
        {wizard.step === "options" && (
          <OptionsStep
            settings={wizard.settings}
            onChange={wizard.changeSettings}
            destination={destination.label}
            onChooseDestination={destination.choose}
            preview={wizard.preview}
            previewing={wizard.previewing}
            error={wizard.error}
            trashSize={wizard.trashSize}
            vaultLocked={wizard.vaultLocked}
            onUnlockVault={onUnlockVault}
          />
        )}
        {wizard.step === "passwords" && request && (
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
        {wizard.step === "report" && wizard.outcome && (
          <ReportStep
            result={wizard.outcome.result}
            passwordsAsked={wizard.outcome.passwordsAsked}
            onGoToItem={(item) => void report.goToItem(item)}
            notice={report.notice}
          />
        )}
      </Dialog>
      {/* Outside the wizard in the React tree too: its keys must not reach the wizard. */}
      {destination.picker}
    </>
  );
}
