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

/**
 * The groups of "Per tipo": the source types of Royal TSX that become
 * connections, folders and credentials; every other type is "Altri tipi".
 * Only a grouping of the report for the eye: the import itself is Rust's.
 */
const ROYAL_GROUPS: Readonly<Record<string, NodeKind>> = {
  RoyalSSHConnection: "ssh",
  RoyalSecureGateway: "ssh",
  RoyalRDSConnection: "rdp",
  RoyalDocument: "folder",
  RoyalFolder: "folder",
  RoyalCredential: "credential",
};

const GROUP_ORDER: readonly NodeKind[] = ["ssh", "rdp", "folder", "credential", "unsupported"];

/** The objects read per group, the empty groups left out. */
export function typeGroups(result: ImportResult): { kind: NodeKind; count: number }[] {
  const groups = new Map<NodeKind, number>();
  for (const [sourceType, counts] of Object.entries(result.report.counts)) {
    const kind = ROYAL_GROUPS[sourceType] ?? "unsupported";
    groups.set(kind, (groups.get(kind) ?? 0) + counts.source);
  }
  return GROUP_ORDER.flatMap((kind) => {
    const count = groups.get(kind) ?? 0;
    return count > 0 ? [{ kind, count }] : [];
  });
}

/**
 * The warnings of the report: those of the importer and of the insertion,
 * the secrets left in the vault, then the passwords skipped in step 3 and the
 * trash left out (DEC-016.8).
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
  const { skipped } = outcomeTotals(result);
  if (skipped > 0) warnings.push(plural(skipped, report.trashSkipped));
  return warnings;
}
