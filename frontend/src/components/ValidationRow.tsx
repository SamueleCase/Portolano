import { texts } from "../i18n/it";
import { type Severity, SeverityMarker } from "./SeverityMarker";
import styles from "./ValidationRow.module.css";
import { visibleText } from "./visibleText";

interface ValidationRowProps {
  severity: Severity;
  /** The readable path of the node, "Produzione / web01". */
  path: string;
  message: string;
  /** Every row leads to its node, errors included (NOTE.md). */
  onGoToNode: () => void;
}

/** A problem of the validation on two lines (ambiguity 9, inventario §4.26). */
export function ValidationRow({ severity, path, message, onGoToNode }: ValidationRowProps) {
  return (
    <li className={styles.row}>
      <span className={styles.severity}>
        <SeverityMarker severity={severity} withLabel />
      </span>
      <span className={styles.path}>{visibleText(path)}</span>
      <span className={styles.message}>{visibleText(message)}</span>
      <button type="button" className={styles.go} onClick={onGoToNode}>
        {texts.components.validation.goToNode}
      </button>
    </li>
  );
}
