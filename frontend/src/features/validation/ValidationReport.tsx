import { useId } from "react";

import type { Issue_Serialize } from "../../bindings";
import { Button } from "../../components/Button";
import { CountBadge } from "../../components/CountBadge";
import { SeverityMarker } from "../../components/SeverityMarker";
import { ValidationRow } from "../../components/ValidationRow";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import styles from "./Validation.module.css";
import { SEVERITIES, bySeverity, countBySeverity, validationTitle } from "./validationSummary";

interface ValidationReportProps {
  issues: readonly Issue_Serialize[];
  /** The readable path of a node, "Produzione / web01". */
  pathOf: (id: string) => string;
  /** "Vai al nodo →" on a row: every problem about a node leads to it (NOTE.md). */
  onGoToNode: (id: string) => void;
  /** "Ricontrolla". */
  onRecheck: () => void;
  /** A check is running: "Ricontrolla" waits for it. */
  checking: boolean;
}

/**
 * The problems of the document (inventario §5.14, ambiguity 9): the real
 * count per level, the rows on two lines, "Ricontrolla". A problem without a
 * node, such as the note of a locked vault, has no "Vai al nodo".
 */
export function ValidationReport({
  issues,
  pathOf,
  onGoToNode,
  onRecheck,
  checking,
}: ValidationReportProps) {
  const titleId = useId();
  const counts = countBySeverity(issues);
  return (
    <section className={styles.report} aria-labelledby={titleId}>
      <header className={styles.header}>
        <h2 id={titleId} className={styles.title}>
          {validationTitle(issues)}
        </h2>
        {issues.length > 0 && (
          <ul className={styles.counts} aria-label={texts.validation.counts}>
            {SEVERITIES.filter((severity) => counts[severity] > 0).map((severity) => (
              <li key={severity}>
                <CountBadge severity={severity} count={counts[severity]} />
              </li>
            ))}
          </ul>
        )}
        <Button className={styles.recheck} onClick={onRecheck} loading={checking}>
          {texts.validation.recheck}
        </Button>
      </header>
      {issues.length === 0 ? (
        <p className={styles.none}>{texts.validation.noneHint}</p>
      ) : (
        <ul className={styles.rows} aria-label={texts.validation.list}>
          {bySeverity(issues).map(({ severity, node, code, message }, index) =>
            node === null ? (
              <li key={`${code}-${index}`} className={styles.note}>
                <SeverityMarker severity={severity} withLabel />
                <span className={styles.noteMessage}>{visibleText(message)}</span>
              </li>
            ) : (
              <ValidationRow
                key={`${code}-${node}-${index}`}
                severity={severity}
                path={pathOf(node)}
                message={message}
                onGoToNode={() => onGoToNode(node)}
              />
            ),
          )}
        </ul>
      )}
    </section>
  );
}
