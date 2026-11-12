import { classNames } from "./classNames";
import type { ImportOutcome } from "./importOutcome";
import styles from "./ReportRow.module.css";
import { visibleText } from "./visibleText";

interface ReportRowProps {
  outcome: Exclude<ImportOutcome, "imported">;
  /** "Parziale", "Non supportato", "Saltato". */
  outcomeLabel: string;
  path: string;
  reason: string;
  /** Leads to the node through its `source_id`, when it exists in the document. */
  onGoToNode?: (() => void) | undefined;
}

/** An item to check in the import report (inventario §4.26). */
export function ReportRow({ outcome, outcomeLabel, path, reason, onGoToNode }: ReportRowProps) {
  const body = (
    <>
      <span className={classNames(styles.outcome, styles[outcome])}>{outcomeLabel}</span>
      <span className={styles.detail}>
        <span className={styles.path}>{visibleText(path)}</span>
        <span className={styles.reason}>{visibleText(reason)}</span>
      </span>
    </>
  );
  return (
    <li className={styles.item}>
      {onGoToNode ? (
        <button type="button" className={styles.row} onClick={onGoToNode}>
          {body}
        </button>
      ) : (
        <div className={styles.row}>{body}</div>
      )}
    </li>
  );
}
