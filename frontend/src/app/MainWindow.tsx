import { useEffect } from "react";

import type { AppStatus_Serialize, VaultKind } from "../bindings";
import type { VaultBackend } from "../components/SecretBadge";
import { backgroundControl } from "../design/background/backgroundControl";
import { DeleteConfirm } from "../features/details/DeleteConfirm";
import { useSessionsStore } from "../features/sessions/store";
import { Sidebar } from "../features/tree/Sidebar";
import { useTreeIndex } from "../features/tree/useTreeIndex";
import { useBackendEvents } from "../ipc/events";
import { useWorkspace } from "../state/workspace";
import { DocumentBanner } from "./DocumentBanner";
import styles from "./MainWindow.module.css";
import { Toolbar } from "./Toolbar";
import { useCommandRunner, useShortcuts } from "./useWindowCommands";
import { useWindowActions } from "./windowActions";
import { WorkArea } from "./WorkArea";
import { useWorkTabs } from "./workTabs";

/** The backend of the vault as the badges name it (the memory vault only exists in tests). */
function vaultBackend(kind: VaultKind): VaultBackend {
  return kind === "file" ? "file" : "keychain";
}

/** A session that connects is work in a session: the background calms down (inventario §3.1). */
function useSessionActivity(): void {
  useEffect(
    () =>
      useSessionsStore.subscribe((state, previous) => {
        const connected = state.sessions.some(
          (session) =>
            session.state.state === "connected" &&
            previous.sessions.find((before) => before.id === session.id)?.state.state !==
              "connected",
        );
        if (connected) backgroundControl.noteSessionActivity();
      }),
    [],
  );
}

/**
 * The main window with a document open (inventario §5.1): toolbar, banner of
 * the document, sidebar and work area, with the confirmation of a deletion.
 * The native menu and the shortcuts perform the same commands.
 */
export function MainWindow({ status }: { status: AppStatus_Serialize }) {
  const actions = useWindowActions();
  const index = useTreeIndex().data;
  const tabs = useWorkTabs(index);
  const run = useCommandRunner(actions, tabs);
  const setBanner = useWorkspace((state) => state.setBanner);
  const backend = vaultBackend(status.vault);

  useBackendEvents({
    onModifiedExternally: (fileName) => setBanner({ kind: "modified", fileName }),
    onMissing: (fileName) => setBanner({ kind: "missing", fileName }),
    onMenu: run,
  });
  useShortcuts(run);
  useSessionActivity();

  return (
    <>
      <div className={styles.window}>
        <Toolbar documentName={status.document?.fileName ?? ""} />
        <DocumentBanner />
        <div className={styles.body}>
          <Sidebar backend={backend} />
          {index ? <WorkArea index={index} backend={backend} /> : <main />}
        </div>
      </div>
      <DeleteConfirm />
    </>
  );
}
