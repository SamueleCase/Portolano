import { useEffect, useMemo, useRef, useState } from "react";

import { Button } from "../../components/Button";
import { tabElementId } from "../../components/tabIds";
import { Tabs, type WorkTab, type WorkTabKind } from "../../components/Tabs";
import { createSessionController } from "../../features/sessions/controller";
import { createOutputBus } from "../../features/sessions/outputBus";
import { SessionServicesContext } from "../../features/sessions/sessionServices";
import { sessionDot } from "../../features/sessions/sessionState";
import { SessionView } from "../../features/sessions/SessionView";
import { type Session, useSessionsStore } from "../../features/sessions/store";
import { gallery } from "../../i18n/gallery";
import { sessionsGallery } from "../../i18n/sessions";
import type { SessionKind } from "../../ipc/sessions";
import { FakeSessionTransport } from "../../test/fakeSessions";
import styles from "../Gallery.module.css";
import { GallerySection } from "../GallerySection";

type FakeNode = keyof typeof sessionsGallery.names;

/** The names and summaries of the fake nodes, by id. */
const names: Readonly<Record<string, string>> = sessionsGallery.names;
const summaries: Readonly<Record<string, string>> = sessionsGallery.summaries;

const PANEL = "gallery-sessions";
const SIZE = { cols: 80, rows: 24 };

/** The sessions the section opens at once: one tab of each kind. */
const OPENED_AT_START: readonly FakeNode[] = ["node-ssh-ok", "node-rdp", "node-windows-app"];

/** What each button of the section opens. */
const OPENERS: readonly (readonly [string, FakeNode])[] = [
  [sessionsGallery.open.ssh, "node-ssh-ok"],
  [sessionsGallery.open.refused, "node-ssh-refused"],
  [sessionsGallery.open.rdp, "node-rdp"],
  [sessionsGallery.open.rdpFailed, "node-rdp-failed"],
  [sessionsGallery.open.windowsApp, "node-windows-app"],
];

const tabKind: Record<SessionKind, WorkTabKind> = {
  terminal: "ssh",
  "rdp-external": "rdp",
  external: "external",
  "rdp-embedded": "rdp",
};

function workTab(session: Session): WorkTab {
  return {
    id: session.id,
    kind: tabKind[session.kind],
    title: names[session.nodeId] ?? session.nodeId,
    dot: sessionDot(session.state),
    closable: true,
  };
}

/**
 * The session tabs on the fake transport (SPEC-UI §5.4 "Sessioni"): a
 * terminal that echoes and ends with `exit N`, a refused SSH that "Riprova"
 * brings back, FreeRDP running or failed with its log, Windows App.
 */
export function SessionsSection() {
  const services = useMemo(() => {
    const output = createOutputBus();
    const controller = createSessionController({
      transport: new FakeSessionTransport(),
      store: useSessionsStore.getState,
      output,
    });
    return { controller, output, openLink: () => undefined };
  }, []);
  const sessions = useSessionsStore((state) => state.sessions);
  const [chosen, setChosen] = useState<string>();
  // Once, even if the effect runs twice (strict mode in development).
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    for (const node of OPENED_AT_START) {
      void services.controller.open({ nodeId: node, size: SIZE });
    }
  }, [services]);

  const selected = sessions.find((session) => session.id === chosen) ?? sessions[0];
  const open = (node: FakeNode) =>
    void services.controller.open({ nodeId: node, size: SIZE }).then(setChosen);

  return (
    <GallerySection id="sessions" title={gallery.section.sessions}>
      <SessionServicesContext value={services}>
        <div className={styles.column}>
          <div className={styles.sample}>
            {OPENERS.map(([label, node]) => (
              <Button key={node} variant="small" onClick={() => open(node)}>
                {label}
              </Button>
            ))}
          </div>
          <Tabs
            label={sessionsGallery.tabs}
            tabs={sessions.map(workTab)}
            selectedId={selected?.id ?? ""}
            onSelect={setChosen}
            onClose={(id) => void services.controller.close(id)}
            panelId={PANEL}
          />
          <div
            id={PANEL}
            role="tabpanel"
            aria-labelledby={selected ? tabElementId(PANEL, selected.id) : undefined}
            className={styles.terminalBox}
          >
            {selected ? (
              <SessionView
                key={selected.id}
                session={selected}
                summary={summaries[selected.nodeId] ?? ""}
              />
            ) : (
              <p className={styles.note}>{sessionsGallery.empty}</p>
            )}
          </div>
        </div>
      </SessionServicesContext>
    </GallerySection>
  );
}
