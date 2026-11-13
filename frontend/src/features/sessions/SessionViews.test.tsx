import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { FakeSessionTransport } from "../../test/fakeSessions";
import { createSessionController } from "./controller";
import { createOutputBus } from "./outputBus";
import { SessionServicesContext } from "./sessionServices";
import { SessionView } from "./SessionView";
import { type Session, useSessionsStore } from "./store";

// xterm.js needs a real layout engine: a stand-in shows what the session
// sends to the terminal; the e2e runs the real one in WebKit.
vi.mock("../../components/terminal/Terminal", async () => {
  const { useEffect, useState } = await import("react");
  const { texts: shown } = await import("../../i18n/it");
  return {
    Terminal: ({ io }: { io: import("../../components/terminal/terminalIo").TerminalIo }) => {
      const [text, setText] = useState("");
      useEffect(
        () => io.subscribe((data) => setText((before) => before + new TextDecoder().decode(data))),
        [io],
      );
      return (
        <div role="group" aria-label={shown.components.terminal.label}>
          {text}
        </div>
      );
    },
  };
});

const { actions, rdp, external, status } = texts.sessions;

function session(overrides: Partial<Session>): Session {
  return {
    id: "session-1",
    nodeId: "node-rdp",
    kind: "rdp-external",
    state: { state: "connected" },
    startedAt: Date.now() - 12_000,
    ...overrides,
  };
}

/** Renders the view of `shown` with a controller whose calls are recorded. */
function renderView(shown: Session) {
  const output = createOutputBus();
  const controller = createSessionController({
    transport: new FakeSessionTransport(),
    store: useSessionsStore.getState,
    output,
  });
  const calls = {
    retry: vi.spyOn(controller, "retry").mockResolvedValue("session-2"),
    disconnect: vi.spyOn(controller, "disconnect").mockResolvedValue(),
    close: vi.spyOn(controller, "close").mockResolvedValue(),
  };
  render(
    <SessionServicesContext value={{ controller, output, openLink: () => undefined }}>
      <SessionView session={shown} summary={"EXAMPLE\\operatore@192.0.2.10:3389"} />
    </SessionServicesContext>,
  );
  return calls;
}

describe("the RDP tab", () => {
  it("shows that FreeRDP runs in its own window, with the duration", () => {
    renderView(session({}));

    expect(screen.getByRole("heading", { name: rdp.title })).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(status.rdpRunning);
    expect(screen.getByText(/^00:00:1\d$/)).toBeInTheDocument();
  });

  it("stops the duration when the program is over", () => {
    const startedAt = Date.now() - 3_600_000;

    renderView(
      session({ state: { state: "ended", code: 0 }, startedAt, endedAt: startedAt + 65_000 }),
    );

    expect(screen.getByText("00:01:05")).toBeInTheDocument();
  });

  it("disconnects while FreeRDP runs", async () => {
    const calls = renderView(session({}));

    await userEvent.click(screen.getByRole("button", { name: actions.disconnect }));

    expect(calls.disconnect).toHaveBeenCalledWith("session-1");
  });

  it("shows the log of a failure with its control characters visible", async () => {
    renderView(
      session({ state: { state: "failed", code: 131, reason: "[ERROR] rifiutata \u001b[31m" } }),
    );

    await userEvent.click(screen.getByRole("button", { name: actions.showLog }));

    expect(screen.getByLabelText(rdp.log)).toHaveTextContent("[ERROR] rifiutata \\u{1b}[31m");
    expect(screen.getByRole("status")).toHaveTextContent(
      interpolate(status.failedWithCode, { code: 131 }),
    );
  });

  it("retries a failed connection", async () => {
    const calls = renderView(
      session({ state: { state: "failed", code: 131, reason: "rifiutata" } }),
    );

    await userEvent.click(screen.getByRole("button", { name: actions.retry }));

    expect(calls.retry).toHaveBeenCalledWith("session-1", { size: { cols: 80, rows: 24 } });
  });
});

describe("the Windows App tab", () => {
  it("says that Windows App has the connection and only offers to close", async () => {
    const calls = renderView(session({ kind: "external" }));

    expect(screen.getByRole("heading", { name: external.title })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: actions.disconnect })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: actions.close }));
    expect(calls.close).toHaveBeenCalledWith("session-1");
  });
});

describe("the SSH tab", () => {
  it("shows the terminal under the state of the connection", () => {
    renderView(session({ kind: "terminal", nodeId: "node-ssh-ok" }));

    expect(screen.getByRole("group", { name: texts.components.terminal.label })).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(status.connected);
  });

  it("names the exit code once the program has ended", () => {
    renderView(session({ kind: "terminal", state: { state: "ended", code: 3 } }));

    expect(screen.getByRole("status")).toHaveTextContent(
      interpolate(status.endedWithCode, { code: 3 }),
    );
    expect(screen.getByRole("button", { name: actions.close })).toBeInTheDocument();
  });

  it("shows the summary of the connection as text", () => {
    renderView(session({ kind: "terminal" }));

    expect(screen.getByText("EXAMPLE\\operatore@192.0.2.10:3389")).toBeInTheDocument();
  });
});
