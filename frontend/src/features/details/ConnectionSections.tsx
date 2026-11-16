import { useQuery } from "@tanstack/react-query";

import { CommandPreview } from "../../components/CommandPreview";
import { GroupHeader } from "../../components/GroupHeader";
import type { NodeKind } from "../../components/nodeKind";
import { RouteChain } from "../../components/RouteChain";
import type { VaultBackend } from "../../components/SecretBadge";
import { Spinner } from "../../components/Spinner";
import { visibleText } from "../../components/visibleText";
import { presentFailure } from "../../ipc/errors";
import { texts } from "../../i18n/it";
import { connectPreviewQuery } from "../../ipc/queries";
import styles from "./Details.module.css";

/**
 * "Mostra comando" (inventario §5.2): the command of the real plan
 * (`connect_preview`), with the secrets as `<vault:…>` references, and the
 * warnings of the plan.
 */
export function CommandSection({ id, backend }: { id: string; backend: VaultBackend }) {
  const t = texts.details.command;
  const preview = useQuery({ ...connectPreviewQuery(id), staleTime: 0 });
  return (
    <section className={styles.section} aria-label={t.heading}>
      {preview.isPending && (
        <p className={styles.note} role="status">
          <Spinner /> {t.loading}
        </p>
      )}
      {preview.isError && (
        <p className={styles.problem} role="alert">
          {visibleText(presentFailure(preview.error).message)}
        </p>
      )}
      {preview.isSuccess && (
        <>
          <CommandPreview caption={t.caption[backend]} command={preview.data.command} />
          {preview.data.warnings.length > 0 && (
            <ul className={styles.warnings} aria-label={t.warnings}>
              {preview.data.warnings.map((warning) => (
                <li key={warning}>{visibleText(warning)}</li>
              ))}
            </ul>
          )}
        </>
      )}
    </section>
  );
}

/** The route of a connection with jump hosts (inventario §4.21): "tu → jumps → host". */
export function RouteSection({ stops, kind }: { stops: readonly string[]; kind: NodeKind }) {
  const label = texts.components.route.label;
  const chain = stops.map((stop, index) =>
    index === stops.length - 1 ? { label: stop, kind } : { label: stop },
  );
  return (
    <section className={styles.section} aria-label={label}>
      <GroupHeader>{label}</GroupHeader>
      <RouteChain variant="route" stops={chain} />
    </section>
  );
}
