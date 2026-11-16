import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { walk } from "../../test/fakeBackendModel";
import { idOf } from "../../test/fakeBackendFixtures";
import { renderWithQueries } from "../../test/render";
import { type LoginAnswer, useLoginPrompt } from "./useLoginPrompt";

const t = texts.dialogs.login;
const SENTINEL = "sentinel-login-secret-77aa";

function Connect({ id, onAnswer }: { id: string; onAnswer: (answer: LoginAnswer) => void }) {
  const { askLogin, dialog } = useLoginPrompt();
  return (
    <>
      <button type="button" onClick={() => void askLogin(id).then(onAnswer)}>
        connetti
      </button>
      {dialog}
    </>
  );
}

async function ask(name: string) {
  installFakeBackend();
  const onAnswer = vi.fn<(answer: LoginAnswer) => void>();
  const { unmount } = renderWithQueries(<Connect id={await idOf(name)} onAnswer={onAnswer} />);
  await userEvent.click(screen.getByRole("button", { name: "connetti" }));
  return Object.assign(onAnswer, { unmount });
}

describe("useLoginPrompt", () => {
  it("needs no login when the credential is not asked at launch", async () => {
    const onAnswer = await ask("web01");

    await waitFor(() => expect(onAnswer).toHaveBeenCalledWith({}));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("returns the login typed and keeps nothing of it", async () => {
    const onAnswer = await ask("kiosk");
    await userEvent.type(await screen.findByLabelText(t.user), "operatore");
    await userEvent.type(screen.getByLabelText(t.domain), "ACME");
    await userEvent.type(screen.getByLabelText(t.password), SENTINEL);

    await userEvent.click(screen.getByRole("button", { name: t.connect }));

    await waitFor(() =>
      expect(onAnswer).toHaveBeenCalledWith({
        login: { username: "operatore", domain: "ACME", password: SENTINEL },
      }),
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(document.body.innerHTML).not.toContain(SENTINEL);
  });

  it("asks no domain for SSH", async () => {
    const state = initialFakeState();
    const db01 = [...walk(state.document?.nodes ?? [])].find(({ node }) => node.name === "db01");
    if (db01) db01.node.credential = { mode: "prompt" };
    installFakeBackend(state);
    renderWithQueries(<Connect id={await idOf("db01")} onAnswer={() => undefined} />);

    await userEvent.click(screen.getByRole("button", { name: "connetti" }));

    expect(await screen.findByLabelText(t.user)).toBeInTheDocument();
    expect(screen.queryByLabelText(t.domain)).not.toBeInTheDocument();
  });

  it("answers null when it is unmounted while asking", async () => {
    const onAnswer = await ask("kiosk");
    await screen.findByRole("dialog", { name: t.title });

    onAnswer.unmount();

    await waitFor(() => expect(onAnswer).toHaveBeenCalledWith(null));
  });

  it("answers null when the user gives up", async () => {
    const onAnswer = await ask("kiosk");

    await userEvent.click(await screen.findByRole("button", { name: texts.dialogs.cancel }));

    await waitFor(() => expect(onAnswer).toHaveBeenCalledWith(null));
  });
});
