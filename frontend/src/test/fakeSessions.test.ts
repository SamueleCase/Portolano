import { afterEach, describe, expect, it } from "vitest";

import type { SessionState } from "../ipc/sessions";
import { FakeSessionTransport, workingConnection } from "./fakeSessions";

const SIZE = { cols: 80, rows: 24 };

/** A node of the fake document, which `fakeScenarios` does not name. */
const NODE = "00000000-0000-4000-8000-000000000104";

/** Handlers that record the states and decode the output. */
function recorder() {
  const states: SessionState[] = [];
  let output = "";
  return {
    states,
    output: () => output,
    handlers: {
      onOutput: (bytes: Uint8Array) => {
        output += new TextDecoder().decode(bytes);
      },
      onState: (state: SessionState) => states.push(state),
    },
  };
}

/** Lets the scenario of a session play: it starts on a timer. */
const played = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("FakeSessionTransport", () => {
  afterEach(() => {
    delete window.__portolanoFakeTraffic;
  });

  it("refuses a node without a scenario", async () => {
    const transport = new FakeSessionTransport({});

    await expect(transport.open({ nodeId: NODE, size: SIZE }, recorder().handlers)).rejects.toThrow(
      "inesistente",
    );
  });

  it("plays the fallback scenario for a node without its own", async () => {
    const transport = new FakeSessionTransport({}, workingConnection);
    const session = recorder();

    const opened = await transport.open({ nodeId: NODE, size: SIZE }, session.handlers);
    await played();

    expect(opened.kind).toBe("terminal");
    expect(session.states.at(-1)).toEqual({ state: "connected" });
    expect(session.output()).toContain("Last login");
  });

  it("writes down its answer, the states and the output for the sentinel e2e", async () => {
    window.__portolanoFakeTraffic = [];
    const transport = new FakeSessionTransport({}, workingConnection);

    await transport.open({ nodeId: NODE, size: SIZE }, recorder().handlers);
    await played();

    const traffic = window.__portolanoFakeTraffic.join("\n");
    expect(traffic).toMatch(/^answer open_session: .*"kind":"terminal"/m);
    expect(traffic).toContain('state: {"state":"connected"}');
    expect(traffic).toMatch(/^output: Last login/m);
  });
});
