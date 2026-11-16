import { useQuery } from "@tanstack/react-query";

import { isConnection } from "../../components/nodeKind";
import type { VaultBackend } from "../../components/SecretBadge";
import { visibleText } from "../../components/visibleText";
import { presentFailure } from "../../ipc/errors";
import { texts } from "../../i18n/it";
import { nodeDetailsQuery } from "../../ipc/queries";
import { useWorkspace } from "../../state/workspace";
import type { TreeEntry } from "../tree/treeModel";
import { CommandSection, RouteSection } from "./ConnectionSections";
import { CredentialSection } from "./CredentialSection";
import { DetailHeader } from "./DetailHeader";
import styles from "./Details.module.css";
import { InfoSection, UnsupportedNotice, UsageSection } from "./InfoSections";
import { routeOf } from "./route";
import { SecretsSection } from "./SecretsSection";

interface DetailPanelProps {
  entry: TreeEntry;
  backend: VaultBackend;
}

/**
 * The detail of a node (inventario §5.2, §5.3): its head and actions, the
 * command of its plan, its route, its effective credential, its secrets, who
 * uses it and the rest of its data. A skeleton while it loads.
 */
export function DetailPanel({ entry, backend }: DetailPanelProps) {
  const { id } = entry.node;
  const details = useQuery(nodeDetailsQuery(id));
  const commandShown = useWorkspace((state) => state.commandShown);
  const route = routeOf(details.data);

  if (details.isPending) {
    return (
      <div
        className={styles.skeleton}
        role="status"
        aria-busy="true"
        aria-label={texts.details.loading}
      >
        <span className={styles.boneTitle} />
        <span className={styles.bone} />
        <span className={styles.bone} />
      </div>
    );
  }
  if (details.isError) {
    return (
      <p role="alert" className={styles.problem}>
        {texts.details.failed}: {visibleText(presentFailure(details.error).message)}
      </p>
    );
  }
  const data = details.data;
  return (
    <div className={styles.detail}>
      <DetailHeader details={data} kind={entry.kind} descendants={entry.node.descendants} />
      {isConnection(entry.kind) && commandShown && <CommandSection id={id} backend={backend} />}
      {route && <RouteSection stops={route} kind={entry.kind} />}
      <UnsupportedNotice details={data} />
      <CredentialSection details={data} backend={backend} />
      <SecretsSection details={data} backend={backend} />
      <UsageSection details={data} />
      <InfoSection details={data} backend={backend} />
    </div>
  );
}
