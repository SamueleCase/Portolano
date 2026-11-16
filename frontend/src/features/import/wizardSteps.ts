import type { Step } from "../../components/Stepper";
import { texts } from "../../i18n/it";

/** The four steps of the wizard (inventario §5.13). */
export type WizardStep = "file" | "options" | "passwords" | "report";

const STEPS: readonly WizardStep[] = ["file", "options", "passwords", "report"];

/**
 * The stepper at `current`. The passwords step reads as skipped (struck
 * through) when it is not going to ask anything: "Chiedi le password" off or
 * a file without passwords.
 */
export function wizardSteps(current: WizardStep, passwordsSkipped: boolean): Step[] {
  const at = STEPS.indexOf(current);
  return STEPS.map((step, index) => ({
    label: texts.importWizard.steps[step],
    state:
      step === "passwords" && passwordsSkipped
        ? "skipped"
        : index < at
          ? "done"
          : index === at
            ? "current"
            : "upcoming",
  }));
}
