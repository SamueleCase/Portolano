import { type ReactNode, useRef, useState } from "react";

import { Stepper } from "../../components/Stepper";
import { FileStep } from "../../features/import/FileStep";
import { readableImportPath } from "../../features/import/importPath";
import { type ImportSettings, OptionsStep } from "../../features/import/OptionsStep";
import { PasswordStep, type SecretFieldElement } from "../../features/import/PasswordStep";
import { ReportStep } from "../../features/import/ReportStep";
import { ReportActions, WizardFooter } from "../../features/import/WizardActions";
import { type WizardStep, wizardSteps } from "../../features/import/wizardSteps";
import { ValidationReport } from "../../features/validation/ValidationReport";
import { gallery } from "../../i18n/gallery";
import { interpolate } from "../../i18n/format";
import { importGallery } from "../../i18n/import";
import { texts } from "../../i18n/it";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";
import {
  galleryIssues,
  galleryKeyRequest,
  galleryLockedIssues,
  galleryPaths,
  galleryPreview,
  galleryRequest,
  galleryRequests,
  galleryResult,
} from "./importValidationFixtures";
import sectionStyles from "./ImportValidationSection.module.css";

const nothing = () => undefined;

/** A step as the wizard shows it, at the width of its dialog, with the stepper and the footer. */
function WizardFrame({
  step,
  footer,
  children,
}: {
  step: WizardStep;
  footer?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className={step === "report" ? sectionStyles.report : sectionStyles.wizard}>
      <Stepper steps={wizardSteps(step, false)} />
      {children}
      {footer && <div className={sectionStyles.footer}>{footer}</div>}
    </div>
  );
}

/**
 * The steps of the import wizard and the validation tab on fixed data
 * (SPEC-UI §5.4): their flows with the backend are tested by Vitest on the
 * fake backend. "Vai al nodo" here only says where it would lead.
 */
export function ImportValidationSection() {
  const [settings, setSettings] = useState<ImportSettings>({
    into: null,
    onConflict: "update",
    includeTrash: false,
    askPasswords: true,
  });
  const [wentTo, setWentTo] = useState<string | null>(null);
  const passwordField = useRef<SecretFieldElement>(null);
  const keyField = useRef<SecretFieldElement>(null);
  const pathOf = (id: string) => galleryPaths[id] ?? id;
  const fileFooter = <WizardFooter step="file" applying={false} canGoOn onCancel={nothing} />;
  const passwordFooter = (
    <WizardFooter step="passwords" applying={false} onSkip={nothing} onSkipAll={nothing} />
  );

  return (
    <GallerySection id="importValidation" title={gallery.section.importValidation}>
      <div className={styles.wide}>
        <Specimen label={importGallery.fileRead}>
          <WizardFrame step="file" footer={fileFooter}>
            <FileStep
              fileName="sede.rtsz"
              reading={{ state: "readable", objects: 17 }}
              onChoose={nothing}
              choosing={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.fileUnreadable}>
          <WizardFrame
            step="file"
            footer={
              <WizardFooter step="file" applying={false} canGoOn={false} onCancel={nothing} />
            }
          >
            <FileStep
              fileName="illeggibile.rtsz"
              reading={{
                state: "unreadable",
                message: "errore di I/O su 'illeggibile.rtsz': Permission denied (os error 13)",
              }}
              onChoose={nothing}
              choosing={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.options}>
          <WizardFrame
            step="options"
            footer={<WizardFooter step="options" applying={false} canGoOn onBack={nothing} />}
          >
            <OptionsStep
              settings={settings}
              onChange={(change) => setSettings({ ...settings, ...change })}
              destination={texts.importWizard.options.root}
              importedFolder={galleryResult.report.document_name}
              onChooseDestination={nothing}
              preview={galleryPreview}
              previewing={false}
              error={null}
              passwordsDropped={false}
              trashSize={galleryResult.report.trash_items}
              vaultLocked={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.password}>
          <WizardFrame step="passwords" footer={passwordFooter}>
            <PasswordStep
              request={galleryRequest}
              position={1}
              total={galleryRequests.length}
              entered={1}
              skipped={0}
              fieldRef={passwordField}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.privateKey}>
          <WizardFrame step="passwords" footer={passwordFooter}>
            <PasswordStep
              request={galleryKeyRequest}
              position={0}
              total={1}
              entered={0}
              skipped={0}
              fieldRef={keyField}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.report}>
          <WizardFrame step="report">
            <ReportStep
              result={galleryResult}
              passwordsAsked={galleryRequests.length}
              onGoToItem={(item) => setWentTo(readableImportPath(item.path))}
              notice={null}
              actions={
                <ReportActions
                  onExport={nothing}
                  exporting={false}
                  onGoToTree={() => setWentTo(galleryResult.report.document_name)}
                />
              }
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.validation}>
          <ValidationReport
            issues={galleryIssues}
            pathOf={pathOf}
            onGoToNode={(id) => setWentTo(pathOf(id))}
            onRecheck={nothing}
            checking={false}
          />
        </Specimen>
        <Specimen label={importGallery.validationLocked}>
          <ValidationReport
            issues={galleryLockedIssues}
            pathOf={pathOf}
            onGoToNode={(id) => setWentTo(pathOf(id))}
            onRecheck={nothing}
            checking={false}
          />
        </Specimen>
        <Specimen label={importGallery.validationClean}>
          <ValidationReport
            issues={[]}
            pathOf={pathOf}
            onGoToNode={nothing}
            onRecheck={nothing}
            checking={false}
          />
        </Specimen>
        <p role="status" className={sectionStyles.wentTo}>
          {wentTo && interpolate(importGallery.wentTo, { target: wentTo })}
        </p>
      </div>
    </GallerySection>
  );
}
