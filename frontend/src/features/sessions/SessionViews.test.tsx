import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { interpolate } from "../../i18n/format";
import { IpcFailure } from "../../ipc/result";
import { texts } from "../../i18n/it";
import { FakeSessionTransport } from "../../test/fakeSessions";
import { createSessionController } from "./controller";
import { createOutputBus } from "./outputBus";
import { type SessionServices, SessionServicesContext } from "./sessionServices";
import { SessionView } from "./SessionView";
import { type Session, useSessionsStore } from "./store";

// xterm.js needs a real layout engine: a stand-in shows what the session
// sends to the terminal; the e2e runs the real one in WebKit.
vi.mock("../../components/terminal/Terminal", async () => {
  const { useEffect, useState } = await import("react");
  return {
    Terminal: ({
      io,
      label,
    }: {
      io: import("../../components/terminal/terminalIo").TerminalIo;
      label: string;
    }) => {
      const [text, setText] = useState("");
      useEffect(
        () => io.subscribe((data) => setText((before) => before + new TextDecoder().decode(data))),
        [io],
      );
      return (
        <div role="group" aria-label={label}>
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

afterEach(() => {
  vi.useRealTimers();
});

/** Renders the view of `shown` with a controller whose calls are recorded. */
function renderView(
  shown: Session,
  extra: Partial<SessionServices> = {},
  route?: readonly string[],
) {
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
    <SessionServicesContext value={{ controller, output, openLink: () => undefined, ...extra }}>
      <SessionView session={shown} summary={"EXAMPLE\\operatore@192.0.2.10:3389"} route={route} />
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

  it("runs the duration every second", () => {
    vi.useFakeTimers({ toFake: ["Date", "setInterval", "clearInterval"] });
    vi.setSystemTime(new Date("2026-11-13T10:00:00Z"));
    renderView(session({ startedAt: Date.now() - 5_000 }));
    expect(screen.getByText("00:00:05")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(3_000);
    });

    expect(screen.getByText("00:00:08")).toBeInTheDocument();
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

  it("offers Chiudi, not Disconnetti, while connecting", () => {
    renderView(session({ state: { state: "connecting" } }));

    expect(screen.getByRole("button", { name: actions.close })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: actions.disconnect })).toBeNull();
  });

  it("starts an action once, however many clicks come while it runs", async () => {
    const calls = renderView(session({}));
    let finish: () => void = () => undefined;
    calls.disconnect.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    );
    const disconnect = screen.getByRole("button", { name: actions.disconnect });

    await userEvent.click(disconnect);
    await userEvent.click(disconnect);

    expect(calls.disconnect).toHaveBeenCalledOnce();
    expect(disconnect).toHaveAttribute("aria-busy", "true");
    act(() => {
      finish();
    });
  });

  it("keeps the log in the page, hidden until asked", () => {
    renderView(session({ state: { state: "failed", code: 131, reason: "rifiutata" } }));
    const toggle = screen.getByRole("button", { name: actions.showLog });
    const log = document.getElementById(toggle.getAttribute("aria-controls") ?? "");

    expect(log).not.toBeNull();
    expect(log).not.toBeVisible();
  });

  it("shows the log of a failure with its control characters visible", async () => {
    renderView(
      session({ state: { state: "failed", code: 131, reason: "[ERROR] rifiutata \u001b[31m" } }),
    );

    await userEvent.click(screen.getByRole("button", { name: actions.showLog }));

    expect(screen.getByRole("figure", { name: rdp.log })).toHaveTextContent(
      "[ERROR] rifiutata \\u{1b}[31m",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      interpolate(status.failedWithCode, { code: 131 }),
    );
  });

  it("retries a failed connection", async () => {
    const calls = renderView(
      session({ state: { state: "failed", code: 131, reason: "rifiutata" } }),
    );

    await userEvent.click(screen.getByRole("button", { name: actions.retry }));

    expect(calls.retry).toHaveBeenCalledWith("session-1", { cols: 80, rows: 24 }, undefined);
  });

  it("asks the host for a login before retrying, and tells it the new session", async () => {
    const login = { username: "operatore", domain: null, password: "digitata" };
    const onRetry = vi.fn(() => Promise.resolve({ login }));
    const onReplaced = vi.fn();
    const calls = renderView(
      session({ state: { state: "failed", code: 131, reason: "rifiutata" } }),
      { onRetry, onReplaced },
    );

    await userEvent.click(screen.getByRole("button", { name: actions.retry }));

    expect(calls.retry).toHaveBeenCalledWith("session-1", { cols: 80, rows: 24 }, login);
    expect(onReplaced).toHaveBeenCalledWith("session-1", "session-2");
  });

  it("does not retry when the host gives up", async () => {
    const calls = renderView(
      session({ state: { state: "failed", code: 131, reason: "rifiutata" } }),
      { onRetry: () => Promise.resolve(null) },
    );

    await userEvent.click(screen.getByRole("button", { name: actions.retry }));

    expect(calls.retry).not.toHaveBeenCalled();
  });

  it("shows why a retry failed", async () => {
    const calls = renderView(
      session({ state: { state: "failed", code: 131, reason: "rifiutata" } }),
    );
    calls.retry.mockRejectedValue(
      new IpcFailure({ code: "vaultLocked", message: "vault bloccato" }),
    );

    await userEvent.click(screen.getByRole("button", { name: actions.retry }));

    expect(await screen.findByRole("alert")).toHaveTextContent("vault bloccato");
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

    expect(
      screen.getByRole("group", {
        name: interpolate(texts.sessions.terminal, {
          summary: "EXAMPLE\\operatore@192.0.2.10:3389",
        }),
      }),
    ).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(status.connected);
  });

  it("names the exit code once the program has ended", () => {
    renderView(session({ kind: "terminal", state: { state: "ended", code: 3 } }));

    expect(screen.getByRole("status")).toHaveTextContent(
      interpolate(status.endedWithCode, { code: 3 }),
    );
    expect(screen.getByRole("button", { name: actions.close })).toBeInTheDocument();
  });

  it("is headed by a group with the route of the connection", () => {
    renderView(session({ kind: "terminal" }), {}, ["bastion.example.com", "web01.example.com"]);

    const header = screen.getByRole("group", { name: texts.sessions.header });
    expect(header).toHaveTextContent("bastion.example.com");
    expect(header).toHaveTextContent("web01.example.com");
  });

  it("shows the summary of the connection as text", () => {
    renderView(session({ kind: "terminal" }));

    expect(screen.getByText("EXAMPLE\\operatore@192.0.2.10:3389")).toBeInTheDocument();
  });
});
