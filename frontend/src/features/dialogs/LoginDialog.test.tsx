import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { LoginRequest } from "../../bindings";
import { texts } from "../../i18n/it";
import { requestLogin } from "../../state/requests";
import { installFakeBackend } from "../../test/fakeBackend";
import { renderInWindow } from "../../test/renderInWindow";
import { LoginDialog } from "./LoginDialog";

const t = texts.dialogs.login;
/** A sentinel password: the dialog must not keep it. */
const SENTINEL = "sentinel-login-secret-81c2";

function requestFor(protocol: "ssh" | "rdp"): LoginRequest {
  return {
    node: "00000000-0000-4000-8000-000000000001",
    path: "/Clienti/kiosk",
    protocol,
    ask_domain: protocol === "rdp",
  };
}

describe("LoginDialog", () => {
  it("hands the login over once and empties its fields", async () => {
    const onLogin = vi.fn();
    render(<LoginDialog request={requestFor("rdp")} onLogin={onLogin} onClose={() => undefined} />);

    await userEvent.type(screen.getByLabelText(t.user), "operatore");
    await userEvent.type(screen.getByLabelText(t.domain), "ACME");
    await userEvent.type(screen.getByLabelText(t.password), SENTINEL);
    await userEvent.click(screen.getByRole("button", { name: t.connect }));

    expect(onLogin).toHaveBeenCalledExactlyOnceWith({
      username: "operatore",
      domain: "ACME",
      password: SENTINEL,
    });
    for (const label of [t.user, t.domain, t.password]) {
      expect(screen.getByLabelText(label)).toHaveValue("");
    }
  });

  it("asks the domain only when the protocol uses one", () => {
    const { rerender } = render(
      <LoginDialog
        request={requestFor("ssh")}
        onLogin={() => undefined}
        onClose={() => undefined}
      />,
    );
    expect(screen.queryByLabelText(t.domain)).not.toBeInTheDocument();

    rerender(
      <LoginDialog
        request={requestFor("rdp")}
        onLogin={() => undefined}
        onClose={() => undefined}
      />,
    );
    expect(screen.getByLabelText(t.domain)).toBeInTheDocument();
  });

  it("answers no login to the window when cancelled", async () => {
    installFakeBackend();
    renderInWindow(<></>);
    let answer: Promise<unknown> = Promise.resolve(undefined);
    act(() => {
      answer = requestLogin(requestFor("ssh"));
    });

    await userEvent.click(await screen.findByRole("button", { name: texts.dialogs.cancel }));

    await expect(answer).resolves.toBeNull();
    await waitFor(() => expect(screen.queryByLabelText(t.password)).not.toBeInTheDocument());
  });
});
