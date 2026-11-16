import type { ImportResult } from "../../bindings";
import type { ImportOutcome } from "../../components/importOutcome";
import type { NodeKind } from "../../components/nodeKind";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";

/** The outcomes in the order of the bar and the tiles (inventario §5.13). */
export const OUTCOMES: readonly ImportOutcome[] = ["imported", "partial", "unsupported", "skipped"];

/** How many source objects ended in each outcome, and how many were read. */
export function outcomeTotals(result: ImportResult): Record<ImportOutcome, number> & {
  read: number;
} {
  const totals = { read: 0, imported: 0, partial: 0, unsupported: 0, skipped: 0 };
  for (const counts of Object.values(result.report.counts)) {
    totals.read += counts.source;
    for (const outcome of OUTCOMES) totals[outcome] += counts[outcome];
  }
  return totals;
}

/** A bar of "Per tipo". */
export interface TypeGroup {
  key: string;
  label: string;
  /** The color of the bar. */
  kind: NodeKind;
  count: number;
}

/** The protocols the interface draws with their own color and name. */
const PROTOCOL_KINDS: Readonly<Record<string, NodeKind>> = { ssh: "ssh", rdp: "rdp" };

/**
 * "Per tipo": the objects read per kind of node, as the importer counts them
 * (DEC-039.1): the connections per protocol, the largest first, then folders,
 * credentials and the other types; the empty groups left out.
 */
export function typeGroups(result: ImportResult): TypeGroup[] {
  const { kinds } = result.report;
  const { types } = texts.importWizard.report;
  const connections = Object.entries(kinds.connections)
    .sort(([a, left], [b, right]) => right - left || a.localeCompare(b))
    .map(([protocol, count]) => {
      const kind = PROTOCOL_KINDS[protocol];
      return kind
        ? { key: protocol, label: types[kind], kind, count }
        : { key: protocol, label: protocol.toUpperCase(), kind: "unsupported" as const, count };
    });
  const others: TypeGroup[] = [
    { key: "folders", label: types.folder, kind: "folder", count: kinds.folders },
    { key: "credentials", label: types.credential, kind: "credential", count: kinds.credentials },
    { key: "unsupported", label: types.unsupported, kind: "unsupported", count: kinds.unsupported },
  ];
  return [...connections, ...others].filter((group) => group.count > 0);
}

/**
 * The warnings of the report: those of the importer and of the insertion,
 * the secrets left in the vault, then the passwords skipped in step 3 and the
 * trash left out (DEC-016.8), with its items and not the trash itself.
 */
export function reportWarnings(result: ImportResult, passwordsAsked: number): string[] {
  const { report } = texts.importWizard;
  const warnings = [
    ...result.report.warnings,
    ...result.warnings,
    ...result.cleanup_failures.map((failure) =>
      interpolate(report.cleanupFailure, { reason: failure.reason }),
    ),
  ];
  const skippedPasswords = passwordsAsked - result.secrets_written;
  if (skippedPasswords > 0) warnings.push(plural(skippedPasswords, report.skippedPasswords));
  const trashItems = result.report.trash_items;
  if (outcomeTotals(result).skipped > 0 && trashItems > 0) {
    warnings.push(plural(trashItems, report.trashSkipped));
  }
  return warnings;
}
