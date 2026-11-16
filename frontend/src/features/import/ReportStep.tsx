import { type ReactNode, useId } from "react";

import type { ImportResult, ReportItem } from "../../bindings";
import { GroupHeader } from "../../components/GroupHeader";
import { MiniBar } from "../../components/MiniBar";
import { StackedBar } from "../../components/StackedBar";
import { StatTile } from "../../components/StatTile";
import { visibleText } from "../../components/visibleText";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./ImportWizard.module.css";
import { ReportItems } from "./ReportItems";
import { OUTCOMES, outcomeTotals, reportWarnings, typeGroups } from "./reportSummary";

interface ReportStepProps {
  result: ImportResult;
  /** The passwords the import asked for: those not written were skipped. */
  passwordsAsked: number;
  onGoToItem: (item: ReportItem) => void;
  /** What "Esporta JSON" or a click on a row has to tell, if anything. */
  notice: string | null;
  /** "Esporta JSON" and "Vai all'albero", at the right of the heading (inventario §5.13). */
  actions: ReactNode;
}

/**
 * Step 4 of the wizard (inventario §5.13): the counts of the outcomes, the
 * types read, the warnings and the items to check. A skipped password makes
 * an item "Parziale"; "Saltati" are only the items of the trash (DEC-016.8).
 * Every text from the file is shown with its control characters visible.
 */
export function ReportStep({
  result,
  passwordsAsked,
  onGoToItem,
  notice,
  actions,
}: ReportStepProps) {
  const warningsId = useId();
  const { report } = texts.importWizard;
  const totals = outcomeTotals(result);
  const groups = typeGroups(result);
  const largest = Math.max(0, ...groups.map((group) => group.count));
  const warnings = reportWarnings(result, passwordsAsked);
  const segments = OUTCOMES.map((outcome) => ({
    outcome,
    label: report.outcomes[outcome],
    count: totals[outcome],
  }));
  const summary = segments.map((segment) => `${segment.label} ${segment.count}`).join(", ");

  return (
    <div className={styles.report}>
      <header className={styles.reportHeader}>
        <div className={styles.reportTitle}>
          <p className={styles.muted}>
            {interpolate(report.completed, { file: visibleText(result.report.source_file) })}
          </p>
          <p className={styles.hero}>
            <span className={styles.heroValue}>{totals.read}</span>
            <span>{plural(totals.read, report.read)}</span>
          </p>
        </div>
        <div className={styles.reportActions}>{actions}</div>
      </header>
      <StackedBar label={interpolate(report.bar, { summary })} segments={segments} />
      <div className={styles.tiles}>
        {segments.map((segment) => (
          <StatTile key={segment.outcome} {...segment} total={totals.read} />
        ))}
      </div>
      <div className={styles.reportColumns}>
        <div className={styles.column}>
          <GroupHeader>{report.byType}</GroupHeader>
          {groups.map((group) => (
            <MiniBar
              key={group.key}
              label={group.label}
              value={group.count}
              max={largest}
              kind={group.kind}
            />
          ))}
          <GroupHeader>
            <span id={warningsId}>{report.warnings}</span>
          </GroupHeader>
          {warnings.length === 0 ? (
            <p className={styles.muted}>{report.noWarnings}</p>
          ) : (
            <ul className={styles.warnings} aria-labelledby={warningsId}>
              {warnings.map((warning, index) => (
                // Two warnings can read the same: the index tells them apart.
                <li key={`${index}-${warning}`}>{visibleText(warning)}</li>
              ))}
            </ul>
          )}
        </div>
        <ReportItems items={result.report.items} onGoToItem={onGoToItem} />
      </div>
      {notice && (
        <p role="status" className={styles.notice}>
          {visibleText(notice)}
        </p>
      )}
    </div>
  );
}
