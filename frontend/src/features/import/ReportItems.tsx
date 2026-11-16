import { useId, useState } from "react";

import type { ReportItem } from "../../bindings";
import { Button } from "../../components/Button";
import { FilterChip } from "../../components/FilterChip";
import { GroupHeader } from "../../components/GroupHeader";
import { ReportRow } from "../../components/ReportRow";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./ImportWizard.module.css";

type Filter = "all" | "partial" | "unsupported";

const FILTERS: readonly Filter[] = ["all", "partial", "unsupported"];

/** The rows shown before "Mostra tutti i N". */
const FIRST_ROWS = 8;

interface ReportItemsProps {
  /** The items not fully imported, as the importer reports them. */
  items: readonly ReportItem[];
  /** A click on a row: the node of the item, through its `source_id`. */
  onGoToItem: (item: ReportItem) => void;
}

/**
 * "Elementi da controllare" of the report (inventario §5.13): the filters,
 * a row per item with its reasons, a click leads to the node. The items of
 * the trash were not imported, so they lead nowhere.
 */
export function ReportItems({ items, onGoToItem }: ReportItemsProps) {
  const titleId = useId();
  const [filter, setFilter] = useState<Filter>("all");
  const [showAll, setShowAll] = useState(false);
  const { report } = texts.importWizard;
  const filtered = items.filter((item) => filter === "all" || item.status === filter);
  const shown = showAll ? filtered : filtered.slice(0, FIRST_ROWS);

  return (
    <section className={styles.column} aria-labelledby={titleId}>
      <GroupHeader>
        <span id={titleId}>{interpolate(report.toCheck, { n: items.length })}</span>
      </GroupHeader>
      {items.length === 0 ? (
        <p className={styles.muted}>{report.nothingToCheck}</p>
      ) : (
        <>
          <div role="group" aria-label={report.filterLabel} className={styles.filters}>
            {FILTERS.map((value) => (
              <FilterChip
                key={value}
                variant="report"
                active={filter === value}
                onToggle={() => setFilter(value)}
              >
                {report.filters[value]}
              </FilterChip>
            ))}
          </div>
          <ul className={styles.items}>
            {shown.map((item) =>
              item.status === "imported" ? null : (
                <ReportRow
                  key={`${item.source_id}-${item.path}`}
                  outcome={item.status}
                  outcomeLabel={report.outcome[item.status]}
                  path={item.path}
                  reason={item.reasons.join(report.reasonSeparator)}
                  onGoToNode={item.status === "skipped" ? undefined : () => onGoToItem(item)}
                />
              ),
            )}
          </ul>
          {shown.length < filtered.length && (
            <Button variant="ghost" className={styles.start} onClick={() => setShowAll(true)}>
              {interpolate(report.showAll, { n: filtered.length })}
            </Button>
          )}
        </>
      )}
    </section>
  );
}
