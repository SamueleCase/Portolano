import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createSessionController } from "../features/sessions/controller";
import { createOutputBus } from "../features/sessions/outputBus";
import { useSessionsStore } from "../features/sessions/store";
import { treeQuery } from "../ipc/queries";
import { createQueryClient } from "../ipc/queryClient";
import { useWorkspace } from "../state/workspace";
import { installFakeBackend } from "../test/fakeBackend";
import { idOf } from "../test/fakeBackendFixtures";
import { FakeSessionTransport, type FakeScenario } from "../test/fakeSessions";
import type { SessionController } from "../features/sessions/controller";
import { IpcFailure } from "../ipc/result";
import type { OpenSessionRequest, SessionHandlers, SessionLogin } from "../ipc/sessions";
import { answerRequest, closeAllRequests, type HostRequest, useRequests } from "../state/requests";
import { useToasts } from "../state/toasts";
import { createConnections, retryingAfterUnlock } from "./connections";

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
    canReadyVault: () => false,
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

/** The failure of a connection that needs the secrets of a locked vault. */
const vaultLocked = () =>
  new IpcFailure({ code: "vaultLocked", message: "il vault è bloccato: sbloccalo" });

/** The fake transport, whose first `locked` openings fail as with the vault locked. */
class LockedVaultTransport extends FakeSessionTransport {
  readonly opened: OpenSessionRequest[] = [];

  constructor(
    scenarios: Record<string, FakeScenario>,
    private locked: number,
  ) {
    super(scenarios);
  }

  override open(request: OpenSessionRequest, handlers: SessionHandlers) {
    this.opened.push(request);
    if (this.locked > 0) {
      this.locked -= 1;
      return Promise.reject(vaultLocked());
    }
    return super.open(request, handlers);
  }
}

/** A login typed at launch (invented). */
const LOGIN: SessionLogin = { username: "operatore", domain: null, password: "prova-di-login" };

/**
 * Answers the dialogs of the window as the user would: the login, and the
 * unlock with `unlock`; returns the kinds of the requests that opened.
 */
function answerDialogs(unlock: boolean): HostRequest["kind"][] {
  const asked: HostRequest["kind"][] = [];
  const seen = new Set<number>();
  const stop = useRequests.subscribe(({ stack }) => {
    for (const { id, request } of stack) {
      if (seen.has(id)) continue;
      seen.add(id);
      asked.push(request.kind);
      queueMicrotask(() => {
        if (request.kind === "login") answerRequest(id, LOGIN);
        if (request.kind === "unlockVault") answerRequest(id, unlock);
      });
    }
  });
  listeners.push(stop);
  return asked;
}

/** The answerers of the test running, removed after it. */
const listeners: (() => void)[] = [];

/** Connections to `kiosk` (credential asked at launch) over `transport`, with every dialog mounted. */
async function kioskOver(locked: number) {
  installFakeBackend();
  const kiosk = await idOf("kiosk");
  const transport = new LockedVaultTransport({ [kiosk]: { kind: "terminal" } }, locked);
  const controller = createSessionController({
    transport,
    store: useSessionsStore.getState,
    output: createOutputBus(),
  });
  const queryClient = createQueryClient();
  await queryClient.fetchQuery(treeQuery);
  const connections = createConnections({
    queryClient,
    controller,
    hasLoginDialog: () => true,
    canReadyVault: () => true,
  });
  return { connections, kiosk, transport, queryClient, controller };
}

describe("the connections with the vault locked (DEC-042)", () => {
  beforeEach(() => {
    useSessionsStore.setState({ sessions: [] });
    useToasts.setState({ toasts: [] });
    useWorkspace.getState().documentSwitched();
  });
  afterEach(() => {
    for (const stop of listeners.splice(0)) stop();
    closeAllRequests();
  });

  it("open without asking the vault when the backend does not ask for it", async () => {
    const { connections, kiosk } = await kioskOver(0);
    const asked = answerDialogs(true);

    await connections.connect(kiosk);

    expect(asked).toEqual(["login"]);
    expect(sessions()).toHaveLength(1);
  });

  it("ask the login once and try again with it once the vault is open", async () => {
    const { connections, kiosk, transport } = await kioskOver(1);
    const asked = answerDialogs(true);

    await connections.connect(kiosk);

    expect(asked).toEqual(["login", "unlockVault"]);
    expect(transport.opened.map((request) => request.login)).toEqual([LOGIN, LOGIN]);
    expect(sessions()).toHaveLength(1);
  });

  it("keep the error and do not try again when the unlock is cancelled", async () => {
    const { connections, kiosk, transport } = await kioskOver(1);
    answerDialogs(false);

    await connections.connect(kiosk);

    expect(transport.opened).toHaveLength(1);
    expect(sessions()).toHaveLength(0);
    expect(useToasts.getState().toasts.map((toast) => toast.kind)).toEqual(["error"]);
  });

  it("make Riprova of a session try again after the unlock, with the same login", async () => {
    const { queryClient, controller: real } = await kioskOver(0);
    const asked = answerDialogs(true);
    const retry = vi.fn<SessionController["retry"]>().mockRejectedValueOnce(vaultLocked());
    retry.mockResolvedValueOnce("sessione-nuova");
    const controller: SessionController = { ...real, retry };
    const again = { size: { cols: 80, rows: 24 }, login: LOGIN };

    const id = await retryingAfterUnlock(controller, queryClient, () => true).retry(
      "vecchia",
      again,
    );

    expect(id).toBe("sessione-nuova");
    expect(asked).toEqual(["unlockVault"]);
    expect(retry.mock.calls).toEqual([
      ["vecchia", again],
      ["vecchia", again],
    ]);
  });
});
