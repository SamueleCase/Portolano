import { useEffect } from "react";

import { GlassPanel } from "../components/GlassPanel";
import type { VaultBackend } from "../components/SecretBadge";
import { tabElementId } from "../components/tabIds";
import { Tabs } from "../components/Tabs";
import { backgroundControl } from "../design/background/backgroundControl";
import { DetailPanel } from "../features/details/DetailPanel";
import { EmptyDocument } from "../features/first-run/EmptyDocument";
import { type Session, useSessionsStore } from "../features/sessions/store";
import type { TreeIndex } from "../features/tree/treeModel";
import { useGoToNode } from "../features/tree/useGoToNode";
import { texts } from "../i18n/it";
import { DETAIL_TAB, useWorkspace, VALIDATION_TAB } from "../state/workspace";
import { useFeatures } from "./features";
import { SessionTab } from "./SessionTab";
import styles from "./WorkArea.module.css";
import { useWindowActions } from "./windowActions";
import { useWorkTabs } from "./workTabs";

const PANEL = "work-panel";

interface WorkAreaProps {
  index: TreeIndex;
  backend: VaultBackend;
}

/**
 * The work area (inventario §5.1, §4.8): the tab bar and the tab in front,
 * which is the detail of the selection, the validation (hook point
 * `validation`) or a session; the empty state for an empty document.
 */
export function WorkArea({ index, backend }: WorkAreaProps) {
  const tabs = useWorkTabs(index);
  const activeTab = useWorkspace((state) => state.activeTab);
  const showTab = useWorkspace((state) => state.showTab);
  const selectedId = useWorkspace((state) => state.selectedId);
  const sessions = useSessionsStore((state) => state.sessions);
  const { closeTab } = useWindowActions();
  const Validation = useFeatures().validation;
  const goToNode = useGoToNode();

  // A tab that went away (a session closed from its own header) leaves the detail in front.
  const active = tabs.some((tab) => tab.id === activeTab) ? activeTab : DETAIL_TAB;
  const session: Session | undefined = sessions.find((candidate) => candidate.id === active);

  // The background calms down while a session is in front (inventario §3.1).
  useEffect(() => {
    backgroundControl.setSessionTabActive(session !== undefined);
  }, [session]);

  if (tabs.length === 0) {
    return (
      <main className={styles.area}>
        <EmptyDocument backend={backend} />
      </main>
    );
  }

  const selected = selectedId === null ? undefined : index.entries.get(selectedId);
  return (
    <main className={styles.area} aria-label={texts.app.work.panel}>
      <Tabs tabs={tabs} selectedId={active} onSelect={showTab} onClose={closeTab} panelId={PANEL} />
      <div
        id={PANEL}
        role="tabpanel"
        aria-labelledby={tabElementId(PANEL, active)}
        className={styles.panelSlot}
      >
        {session ? (
          <SessionTab
            key={session.id}
            session={session}
            entry={index.entries.get(session.nodeId)}
          />
        ) : (
          <GlassPanel className={styles.panel}>
            {active === VALIDATION_TAB && Validation ? (
              <Validation onGoToNode={goToNode} />
            ) : selected ? (
              <DetailPanel key={selected.node.id} entry={selected} backend={backend} />
            ) : (
              <p className={styles.hint}>{texts.app.work.noSelection}</p>
            )}
          </GlassPanel>
        )}
      </div>
    </main>
  );
}
