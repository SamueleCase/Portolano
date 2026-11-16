import { beforeEach, describe, expect, it } from "vitest";

import { createSessionController } from "../features/sessions/controller";
import { createOutputBus } from "../features/sessions/outputBus";
import { useSessionsStore } from "../features/sessions/store";
import { treeQuery } from "../ipc/queries";
import { createQueryClient } from "../ipc/queryClient";
import { useWorkspace } from "../state/workspace";
import { installFakeBackend } from "../test/fakeBackend";
import { idOf } from "../test/fakeBackendFixtures";
import { FakeSessionTransport, type FakeScenario } from "../test/fakeSessions";
import { createConnections } from "./connections";

/** Lets the scenarios of the fake transport play (they start on a timer). */
const played = () => new Promise((resolve) => setTimeout(resolve, 0));

/** The connections of the window over the fake backend and the fake transport. */
async function connectionsFor(scenario: FakeScenario) {
  installFakeBackend();
  const bastion = await idOf("bastion");
  const controller = createSessionController({
    transport: new FakeSessionTransport({ [bastion]: scenario }),
    store: useSessionsStore.getState,
    output: createOutputBus(),
  });
  const queryClient = createQueryClient();
  await queryClient.fetchQuery(treeQuery);
  const connections = createConnections({
    queryClient,
    controller,
    hasLoginDialog: () => false,
    hasUnlockDialog: () => false,
  });
  return { connections, bastion };
}

const sessions = () => useSessionsStore.getState().sessions;

describe("the connections of the window", () => {
  beforeEach(() => {
    useSessionsStore.setState({ sessions: [] });
    useWorkspace.getState().documentSwitched();
  });

  it("opens a session in a new tab, in front", async () => {
    const { connections, bastion } = await connectionsFor({ kind: "terminal" });

    await connections.connect(bastion);

    expect(sessions()).toHaveLength(1);
    expect(useWorkspace.getState().activeTab).toBe(sessions()[0]?.id);
  });

  it("brings in front a session of the node still running instead of opening another", async () => {
    const { connections, bastion } = await connectionsFor({ kind: "terminal" });
    await connections.connect(bastion);
    useWorkspace.getState().showTab("detail");

    await connections.connect(bastion);

    expect(sessions()).toHaveLength(1);
    expect(useWorkspace.getState().activeTab).toBe(sessions()[0]?.id);
  });

  it("opens a session that is over again in its place, and keeps it in front", async () => {
    const { connections, bastion } = await connectionsFor({
      kind: "terminal",
      end: { state: "failed", code: 255, reason: "connessione non riuscita" },
      failures: 1,
    });
    await connections.connect(bastion);
    await played();
    const failed = sessions()[0]?.id;

    await connections.connect(bastion);

    expect(sessions()).toHaveLength(1);
    expect(sessions()[0]?.id).not.toBe(failed);
    expect(useWorkspace.getState().activeTab).toBe(sessions()[0]?.id);
  });

  it("opens one session for two connections asked at once", async () => {
    const { connections, bastion } = await connectionsFor({ kind: "terminal" });

    await Promise.all([connections.connect(bastion), connections.connect(bastion)]);

    expect(sessions()).toHaveLength(1);
  });

  it("remembers the summary of the connection, the title of its tab once the node is gone", async () => {
    const { connections, bastion } = await connectionsFor({ kind: "terminal" });

    await connections.connect(bastion);

    const id = sessions()[0]?.id ?? "";
    expect(useWorkspace.getState().sessionSummaries[id]).toBe("deploy@bastion.example.com:22");
  });
});
