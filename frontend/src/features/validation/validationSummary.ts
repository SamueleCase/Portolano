import type { Issue_Serialize } from "../../bindings";
import type { Severity } from "../../components/SeverityMarker";
import { plural } from "../../i18n/format";
import { texts } from "../../i18n/it";

/** The levels in the order the tab lists them: errors first (inventario §5.14). */
export const SEVERITIES: readonly Severity[] = ["error", "warning", "info"];

/** How many problems of each level `issues` has. */
export function countBySeverity(issues: readonly Issue_Serialize[]): Record<Severity, number> {
  const counts: Record<Severity, number> = { error: 0, warning: 0, info: 0 };
  for (const issue of issues) counts[issue.severity] += 1;
  return counts;
}

/** `issues` with the errors first, then the warnings, then the notes, each in the backend order. */
export function bySeverity(issues: readonly Issue_Serialize[]): Issue_Serialize[] {
  return SEVERITIES.flatMap((severity) => issues.filter((issue) => issue.severity === severity));
}

/** The heading of the tab: "1 problema nel documento", "Nessun problema nel documento". */
export function validationTitle(issues: readonly Issue_Serialize[]): string {
  return issues.length === 0
    ? texts.validation.none
    : plural(issues.length, texts.validation.title);
}
