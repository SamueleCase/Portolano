import { type ReactNode, useRef, useState } from "react";

import { Stepper } from "../../components/Stepper";
import { FileStep } from "../../features/import/FileStep";
import { type ImportSettings, OptionsStep } from "../../features/import/OptionsStep";
import { PasswordStep } from "../../features/import/PasswordStep";
import { ReportStep } from "../../features/import/ReportStep";
import { ValidationReport } from "../../features/validation/ValidationReport";
import { gallery } from "../../i18n/gallery";
import { interpolate } from "../../i18n/format";
import { importGallery } from "../../i18n/import";
import { texts } from "../../i18n/it";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";
import {
  galleryIssues,
  galleryLockedIssues,
  galleryPaths,
  galleryPreview,
  galleryRequest,
  galleryRequests,
  galleryResult,
} from "./importValidationFixtures";
import sectionStyles from "./ImportValidationSection.module.css";

const { steps } = texts.importWizard;

/** The stepper of the wizard at `current` (0–3). */
function WizardSteps({ current }: { current: number }) {
  const labels = [steps.file, steps.options, steps.passwords, steps.report];
  return (
    <Stepper
      steps={labels.map((label, index) => ({
        label,
        state: index < current ? "done" : index === current ? "current" : "upcoming",
      }))}
    />
  );
}

/** A step as the wizard shows it, at the width of its dialog. */
function WizardFrame({ wide = false, children }: { wide?: boolean; children: ReactNode }) {
  return <div className={wide ? sectionStyles.report : sectionStyles.wizard}>{children}</div>;
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
  const passwordField = useRef<HTMLInputElement>(null);
  const pathOf = (id: string) => galleryPaths[id] ?? id;

  return (
    <GallerySection id="importValidation" title={gallery.section.importValidation}>
      <div className={styles.wide}>
        <Specimen label={importGallery.fileRead}>
          <WizardFrame>
            <WizardSteps current={0} />
            <FileStep
              fileName="sede.rtsz"
              reading={{ state: "readable", objects: 17 }}
              onChoose={() => undefined}
              choosing={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.fileUnreadable}>
          <WizardFrame>
            <WizardSteps current={0} />
            <FileStep
              fileName="illeggibile.rtsz"
              reading={{
                state: "unreadable",
                message: "errore di I/O su 'illeggibile.rtsz': Permission denied (os error 13)",
              }}
              onChoose={() => undefined}
              choosing={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.options}>
          <WizardFrame>
            <WizardSteps current={1} />
            <OptionsStep
              settings={settings}
              onChange={(change) => setSettings({ ...settings, ...change })}
              destination={texts.importWizard.options.root}
              onChooseDestination={() => undefined}
              preview={galleryPreview}
              previewing={false}
              error={null}
              trashSize={2}
              vaultLocked={false}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.password}>
          <WizardFrame>
            <WizardSteps current={2} />
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
        <Specimen label={importGallery.report}>
          <WizardFrame wide>
            <WizardSteps current={3} />
            <ReportStep
              result={galleryResult}
              passwordsAsked={galleryRequests.length}
              onGoToItem={(item) => setWentTo(item.path)}
              notice={null}
            />
          </WizardFrame>
        </Specimen>
        <Specimen label={importGallery.validation}>
          <ValidationReport
            issues={galleryIssues}
            pathOf={pathOf}
            onGoToNode={(id) => setWentTo(pathOf(id))}
            onRecheck={() => undefined}
            checking={false}
          />
        </Specimen>
        <Specimen label={importGallery.validationLocked}>
          <ValidationReport
            issues={galleryLockedIssues}
            pathOf={pathOf}
            onGoToNode={(id) => setWentTo(pathOf(id))}
            onRecheck={() => undefined}
            checking={false}
          />
        </Specimen>
        <Specimen label={importGallery.validationClean}>
          <ValidationReport
            issues={[]}
            pathOf={pathOf}
            onGoToNode={() => undefined}
            onRecheck={() => undefined}
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
