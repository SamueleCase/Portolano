import { act, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { texts } from "../../i18n/it";
import { useWorkspace } from "../../state/workspace";
import { AddMenu } from "./AddMenu";
import { renderWithActions } from "./renderWithActions";

const t = texts.tree.add;
const { protocols } = texts.app.toolbar;

function addButton() {
  return screen.getByRole("button", { name: t.label });
}

/** Opens the menu of "+" from the keyboard and returns it. */
async function openWithEnter() {
  addButton().focus();
  await userEvent.keyboard("{Enter}");
  return screen.getByRole("menu", { name: t.label });
}

describe("the «+» of the sidebar", () => {
  beforeEach(() => useWorkspace.getState().documentSwitched());

  it("opens with Enter on its first entry", async () => {
    renderWithActions(<AddMenu />);

    const menu = await openWithEnter();

    expect(
      within(menu)
        .getAllByRole("menuitem")
        .map((item) => item.textContent),
    ).toEqual([
      `${t.newConnection}${texts.app.shortcuts.newConnection}`,
      `${t.newFolder}${texts.app.shortcuts.newFolder}`,
      t.newCredential,
      `${t.import}${texts.app.shortcuts.import}`,
    ]);
    expect(within(menu).getByRole("menuitem", { name: t.newConnection })).toHaveFocus();
  });

  it("opens with Space too", async () => {
    renderWithActions(<AddMenu />);
    addButton().focus();

    await userEvent.keyboard(" ");

    expect(screen.getByRole("menu", { name: t.label })).toBeInTheDocument();
  });

  it("separates the import from the new nodes", async () => {
    renderWithActions(<AddMenu />);

    const menu = await openWithEnter();

    expect(within(menu).getByRole("separator")).toBeInTheDocument();
  });

  it("moves among the entries with the arrows", async () => {
    renderWithActions(<AddMenu />);
    const menu = await openWithEnter();

    await userEvent.keyboard("{ArrowDown}{ArrowDown}{ArrowDown}");

    expect(within(menu).getByRole("menuitem", { name: t.import })).toHaveFocus();
  });

  it("closes with Esc and gives the focus back to «+»", async () => {
    renderWithActions(<AddMenu />);
    await openWithEnter();

    await userEvent.keyboard("{Escape}");

    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(addButton()).toHaveFocus();
  });

  it("asks the protocol of a new connection, under «+»", async () => {
    const { calls } = renderWithActions(<AddMenu />);
    await openWithEnter();

    await userEvent.keyboard("{Enter}");
    const protocolsMenu = screen.getByRole("menu", { name: texts.app.toolbar.newConnectionMenu });
    await userEvent.click(within(protocolsMenu).getByRole("menuitem", { name: protocols.rdp }));

    expect(calls).toEqual(["create rdp"]);
  });

  it("gives the focus back to «+» when the protocol menu closes with Esc", async () => {
    renderWithActions(<AddMenu />);
    await openWithEnter();
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("menuitem", { name: protocols.ssh })).toHaveFocus();

    await userEvent.keyboard("{Escape}");

    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(addButton()).toHaveFocus();
  });

  it.each([
    [t.newFolder, "create folder"],
    [t.newCredential, "create credential"],
    [t.import, "openImport"],
  ])("performs «%s»", async (entry, call) => {
    const { calls } = renderWithActions(<AddMenu />);
    await userEvent.click(addButton());

    await userEvent.click(screen.getByRole("menuitem", { name: entry }));

    expect(calls).toEqual([call]);
  });

  it("opens the protocol menu under «+» for ⌘N and the native menu", () => {
    renderWithActions(<AddMenu />);

    act(() => useWorkspace.getState().setConnectionMenu(true));

    expect(screen.getByRole("menu", { name: texts.app.toolbar.newConnectionMenu })).toBeVisible();
  });

  it("disables an entry whose function is not mounted, with the reason", async () => {
    renderWithActions(<AddMenu />, ["editor"]);
    await userEvent.click(addButton());

    const entry = screen.getByRole("menuitem", { name: t.import });

    expect(entry).toBeDisabled();
    expect(entry).toHaveAccessibleDescription(texts.app.reasons.unavailable);
  });

  it("is disabled, with the reason, when none of its functions is mounted", () => {
    renderWithActions(<AddMenu />, []);

    expect(addButton()).toHaveAttribute("aria-disabled", "true");
    expect(addButton()).toHaveAccessibleDescription(texts.app.reasons.unavailable);
  });
});
