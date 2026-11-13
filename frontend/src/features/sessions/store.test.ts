import { beforeEach, describe, expect, it } from "vitest";

import { type Session, useSessionsStore } from "./store";

const session: Session = {
  id: "session-1",
  nodeId: "node-ssh-ok",
  kind: "terminal",
  state: { state: "connecting" },
  startedAt: 1_000,
};

describe("useSessionsStore", () => {
  beforeEach(() => {
    useSessionsStore.setState({ sessions: [] });
  });

  it("keeps sessions in the order they were opened", () => {
    const { added } = useSessionsStore.getState();

    added(session);
    added({ ...session, id: "session-2" });

    expect(useSessionsStore.getState().sessions.map(({ id }) => id)).toEqual([
      "session-1",
      "session-2",
    ]);
  });

  it("records the exit code with the last state", () => {
    const { added, stateChanged } = useSessionsStore.getState();
    added(session);

    stateChanged("session-1", { state: "ended", code: 3 }, 9_000);

    expect(useSessionsStore.getState().sessions[0]).toMatchObject({
      state: { state: "ended", code: 3 },
      exitCode: 3,
      endedAt: 9_000,
    });
  });

  it("has no end time while the program runs", () => {
    const { added, stateChanged } = useSessionsStore.getState();
    added(session);

    stateChanged("session-1", { state: "connected" }, 9_000);

    expect(useSessionsStore.getState().sessions[0]?.endedAt).toBeUndefined();
  });

  it("keeps the last state of a session that is over", () => {
    const { added, stateChanged } = useSessionsStore.getState();
    added(session);
    stateChanged("session-1", { state: "failed", code: 255, reason: "rifiutata" });

    stateChanged("session-1", { state: "ended", code: 0 });

    expect(useSessionsStore.getState().sessions[0]).toMatchObject({
      state: { state: "failed", code: 255 },
      exitCode: 255,
    });
  });

  it("ignores states of sessions it does not know", () => {
    const { added, stateChanged } = useSessionsStore.getState();
    added(session);

    stateChanged("session-9", { state: "connected" });

    expect(useSessionsStore.getState().sessions).toEqual([session]);
  });

  it("puts a session in the place of another", () => {
    const { added, replaced } = useSessionsStore.getState();
    added(session);
    added({ ...session, id: "session-2" });

    replaced("session-1", { ...session, id: "session-3" });

    expect(useSessionsStore.getState().sessions.map(({ id }) => id)).toEqual([
      "session-3",
      "session-2",
    ]);
  });

  it("does not change the list for a state it ignores", () => {
    const { added, stateChanged } = useSessionsStore.getState();
    added({ ...session, state: { state: "ended", code: 0 } });
    const before = useSessionsStore.getState().sessions;

    stateChanged("session-1", { state: "connected" });
    stateChanged("session-9", { state: "connected" });

    expect(useSessionsStore.getState().sessions).toBe(before);
  });

  it("forgets a removed session", () => {
    const { added, removed } = useSessionsStore.getState();
    added(session);

    removed("session-1");

    expect(useSessionsStore.getState().sessions).toEqual([]);
  });
});
