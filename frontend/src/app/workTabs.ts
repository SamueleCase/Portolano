import { useQuery } from "@tanstack/react-query";

import type { WorkTab, WorkTabKind } from "../components/Tabs";
import { sessionDot } from "../features/sessions/sessionState";
import { useSessionsStore } from "../features/sessions/store";
import type { TreeIndex } from "../features/tree/treeModel";
import { texts } from "../i18n/it";
import { validationQuery } from "../ipc/queries";
import type { SessionKind } from "../ipc/sessions";
import { DETAIL_TAB, useWorkspace, VALIDATION_TAB } from "../state/workspace";
import { useFeatures } from "./features";

const sessionTabKind: Record<SessionKind, WorkTabKind> = {
  terminal: "ssh",
  "rdp-external": "rdp",
  external: "external",
  "rdp-embedded": "rdp",
};

/** The tabs of the work area in their order: the detail, the validation, the sessions. */
export function useWorkTabs(index: TreeIndex | undefined): WorkTab[] {
  const selectedId = useWorkspace((state) => state.selectedId);
  const validationOpen = useWorkspace((state) => state.validationOpen);
  const sessions = useSessionsStore((state) => state.sessions);
  const summaries = useWorkspace((state) => state.sessionSummaries);
  const validation = useFeatures().validation !== undefined;
  const problems = useQuery({ ...validationQuery, enabled: validationOpen && validation }).data;
  const name = (id: string | null) => (id === null ? undefined : index?.entries.get(id)?.node.name);
  const tabs: WorkTab[] = [];
  if (index && index.rootIds.length > 0) {
    tabs.push({
      id: DETAIL_TAB,
      kind: "detail",
      title: name(selectedId) ?? texts.app.work.noSelectionTitle,
      dot: "detail",
      closable: false,
    });
  }
  if (validationOpen && validation) {
    tabs.push({
      id: VALIDATION_TAB,
      kind: "validation",
      title: texts.app.work.validationTitle,
      dot: "validation",
      closable: true,
      problems: problems?.length ?? 0,
    });
  }
  for (const session of sessions) {
    tabs.push({
      id: session.id,
      kind: sessionTabKind[session.kind],
      // A node gone (another document, a deletion) leaves the summary of the
      // connection as the title (DEC-041.7).
      title: name(session.nodeId) ?? summaries[session.id] ?? texts.app.work.sessionTitle,
      dot: sessionDot(session.state),
      closable: true,
    });
  }
  return tabs;
}
