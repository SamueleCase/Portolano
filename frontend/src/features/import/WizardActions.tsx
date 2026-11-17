import { Button } from "../../components/Button";
import { texts } from "../../i18n/it";
import styles from "./ImportWizard.module.css";

/** What the footer of each of steps 1–3 needs. */
export type WizardFooterProps = {
  /** The import runs: its buttons wait, so that a second click starts nothing. */
  applying: boolean;
} & (
  | { step: "file"; canGoOn: boolean; onCancel: () => void }
  | { step: "options"; canGoOn: boolean; onBack: () => void }
  | { step: "passwords"; onSkip: () => void; onSkipAll: () => void }
);

/**
 * The buttons of steps 1–3, the footer of the dialog (inventario §5.13).
 * "Avanti" and "Salva e continua" submit the dialog: Enter does the same.
 */
export function WizardFooter(props: WizardFooterProps) {
  const { actions } = texts.importWizard;
  switch (props.step) {
    case "file":
      return (
        <>
          <Button onClick={props.onCancel}>{actions.cancel}</Button>
          <Button type="submit" variant="primary" disabled={!props.canGoOn}>
            {actions.next}
          </Button>
        </>
      );
    case "options":
      return (
        <>
          <Button className={styles.footerStart} onClick={props.onBack}>
            {actions.back}
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={!props.canGoOn}
            loading={props.applying}
          >
            {actions.next}
          </Button>
        </>
      );
    case "passwords":
      return (
        <>
          <Button onClick={props.onSkip} loading={props.applying}>
            {actions.skip}
          </Button>
          <Button
            variant="ghost"
            className={styles.footerStart}
            onClick={props.onSkipAll}
            loading={props.applying}
          >
            {actions.skipAll}
          </Button>
          <Button type="submit" variant="primary" loading={props.applying}>
            {actions.saveAndContinue}
          </Button>
        </>
      );
  }
}

/** "Esporta JSON" and "Vai all'albero", in the heading of the report (inventario §5.13). */
export function ReportActions({
  onExport,
  exporting,
  onGoToTree,
}: {
  onExport: () => void;
  exporting: boolean;
  onGoToTree: () => void;
}) {
  const { actions } = texts.importWizard;
  return (
    <>
      <Button onClick={onExport} loading={exporting}>
        {actions.exportJson}
      </Button>
      <Button variant="primary" onClick={onGoToTree}>
        {actions.goToTree}
      </Button>
    </>
  );
}
