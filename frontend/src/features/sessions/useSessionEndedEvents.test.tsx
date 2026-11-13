import { emit } from "@tauri-apps/api/event";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { installFakeBackend } from "../../test/fakeBackend";
import { useSessionsStore } from "./store";
import { useSessionEndedEvents } from "./useSessionEndedEvents";

let unmount: () => void = () => undefined;

// The listener stops asynchronously: it must be gone before the shared
// teardown removes the mocked event plugin.
afterEach(async () => {
  unmount();
  await new Promise((resolve) => setTimeout(resolve, 0));
});

beforeEach(() => {
  installFakeBackend();
  useSessionsStore.setState({
    sessions: [
      {
        id: "session-1",
        nodeId: "node-rdp",
        kind: "rdp-external",
        state: { state: "connected" },
        startedAt: 0,
      },
    ],
  });
  unmount = renderHook(() => useSessionEndedEvents()).unmount;
});

describe("useSessionEndedEvents", () => {
  it("records the last state of a known session", async () => {
    await act(() => emit("session-ended", { id: "session-1", state: { state: "ended", code: 0 } }));

    await waitFor(() =>
      expect(useSessionsStore.getState().sessions[0]?.state).toEqual({
        state: "ended",
        code: 0,
      }),
    );
  });

  it("ignores a session it does not know yet", async () => {
    await act(() =>
      emit("session-ended", {
        id: "session-9",
        state: { state: "failed", code: 255, reason: "rifiutata" },
      }),
    );

    expect(useSessionsStore.getState().sessions.map(({ id }) => id)).toEqual(["session-1"]);
    expect(useSessionsStore.getState().sessions[0]?.state).toEqual({ state: "connected" });
  });
});
