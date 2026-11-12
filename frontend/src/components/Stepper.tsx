import { Fragment } from "react";

import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./Stepper.module.css";
import { VisuallyHidden } from "./VisuallyHidden";

/** A step: done, the current one, still to come, or skipped (wizard step 3). */
export interface Step {
  label: string;
  state: "done" | "current" | "upcoming" | "skipped";
}

interface StepperProps {
  steps: readonly Step[];
  /** The editor's steps can be clicked; the wizard's cannot. */
  onSelect?: ((index: number) => void) | undefined;
  /** `editor`: done steps in text color, upcoming ones transparent (inventario §4.25). */
  variant?: "wizard" | "editor";
}

/** The steps of the editor and of the import wizard (inventario §4.25). */
export function Stepper({ steps, onSelect, variant = "wizard" }: StepperProps) {
  return (
    <ol
      aria-label={texts.components.stepper.label}
      className={classNames(styles.stepper, styles[variant])}
    >
      {steps.map((step, index) => {
        const content = (
          <>
            {index + 1} {step.label}
            {step.state !== "current" && step.state !== "upcoming" && (
              <VisuallyHidden> ({texts.components.stepper.states[step.state]})</VisuallyHidden>
            )}
          </>
        );
        const className = classNames(styles.step, styles[step.state]);
        return (
          <Fragment key={step.label}>
            {index > 0 && (
              <li
                className={classNames(
                  styles.line,
                  step.state !== "upcoming" && step.state !== "skipped" && styles.passed,
                )}
                aria-hidden="true"
              />
            )}
            <li aria-current={step.state === "current" ? "step" : undefined}>
              {onSelect ? (
                <button type="button" className={className} onClick={() => onSelect(index)}>
                  {content}
                </button>
              ) : (
                <span className={className}>{content}</span>
              )}
            </li>
          </Fragment>
        );
      })}
    </ol>
  );
}
