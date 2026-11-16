import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { idOf } from "../../test/fakeBackendFixtures";
import { renderWithQueries } from "../../test/render";
import { IconChooser } from "./IconChooser";
import { TreeNodePicker } from "./TreeNodePicker";
import { stubScrollIntoView } from "./scrollStub";

stubScrollIntoView();

const icons = texts.components.iconPicker;

describe("IconChooser", () => {
  it("uses a builtin icon by its name", async () => {
    installFakeBackend();
    const onUse = vi.fn();
    renderWithQueries(
      <IconChooser
        open
        value={null}
        onUse={onUse}
        onClose={() => undefined}
        onError={() => undefined}
      />,
    );

    await userEvent.click(screen.getByRole("radio", { name: texts.icons.database }));
    await userEvent.click(screen.getByRole("button", { name: icons.use }));

    expect(onUse).toHaveBeenCalledWith({ kind: "builtin", name: "database" });
  });

  it("stores a new image and uses it at once", async () => {
    const state = initialFakeState();
    state.dialogs.icon = "/Users/utente-di-prova/icone/server.png";
    installFakeBackend(state);
    const onUse = vi.fn();
    renderWithQueries(
      <IconChooser
        open
        value={null}
        onUse={onUse}
        onClose={() => undefined}
        onError={() => undefined}
      />,
    );

    await userEvent.click(screen.getByRole("radio", { name: icons.sources.image }));
    await userEvent.click(screen.getByRole("button", { name: icons.chooseImage }));

    await waitFor(() =>
      expect(onUse).toHaveBeenCalledWith(expect.objectContaining({ kind: "custom" })),
    );
    expect(state.customIcons).toHaveLength(1);
  });

  it("uses nothing when the native dialog is cancelled", async () => {
    installFakeBackend();
    const onUse = vi.fn();
    const onError = vi.fn();
    renderWithQueries(
      <IconChooser open value={null} onUse={onUse} onClose={() => undefined} onError={onError} />,
    );
    await userEvent.click(screen.getByRole("radio", { name: icons.sources.image }));

    await userEvent.click(screen.getByRole("button", { name: icons.chooseImage }));

    expect(screen.getByRole("dialog", { name: icons.title })).toBeInTheDocument();
    expect(onUse).not.toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
  });
});

describe("TreeNodePicker", () => {
  it("offers the SSH connections as jump hosts, except the node itself", async () => {
    installFakeBackend();
    const web01 = await idOf("web01");
    renderWithQueries(
      <TreeNodePicker
        open
        purpose="jumpHost"
        exclude={web01}
        onChoose={() => undefined}
        onClose={() => undefined}
      />,
    );

    const list = await screen.findByRole("listbox");
    await waitFor(() => expect(within(list).getAllByRole("option")).toHaveLength(3));
    expect(within(list).getByRole("option", { name: /web01/ })).toHaveAttribute(
      "aria-disabled",
      "true",
    );
    expect(within(list).queryByRole("option", { name: /dc01/ })).not.toBeInTheDocument();
    expect(
      screen.getByText(/web01 è escluso perché non può fare da salto a se stesso/),
    ).toBeInTheDocument();
  });
});
