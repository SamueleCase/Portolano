import { plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import type { Severity } from "./SeverityMarker";
import styles from "./StatusBadge.module.css";

/** The counters of the validation: "1 errore", "2 avvisi", "1 nota". */
export function CountBadge({ severity, count }: { severity: Severity; count: number }) {
  return (
    <span className={classNames(styles.badge, styles.count, styles[severity])}>
      {plural(count, texts.components.severityCounts[severity])}
    </span>
  );
}
