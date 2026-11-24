import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { texts } from "../../i18n/it";
import { renderWithActions } from "./renderWithActions";
import { SidebarTools } from "./SidebarTools";

const t = texts.tree.tools;

describe("the tools of the footer of the sidebar", () => {
  it("names its two buttons, Valida and Preferenze", () => {
    renderWithActions(<SidebarTools />);

    const tools = screen.getByRole("group", { name: t.label });

    expect(
      within(tools)
        .getAllByRole("button")
        .map((button) => button.ariaLabel),
    ).toEqual([t.validate, t.preferences]);
  });

  it("opens the validation with Valida", async () => {
    const { calls } = renderWithActions(<SidebarTools />);

    await userEvent.click(screen.getByRole("button", { name: t.validate }));

    expect(calls).toEqual(["openValidation"]);
  });

  it("opens the preferences with Preferenze", async () => {
    const { calls } = renderWithActions(<SidebarTools />);

    await userEvent.click(screen.getByRole("button", { name: t.preferences }));

    expect(calls).toEqual(["openPreferences"]);
  });

  it("describes each button with its tooltip and shortcut", () => {
    renderWithActions(<SidebarTools />);

    expect(screen.getByRole("button", { name: t.preferences })).toHaveAccessibleDescription(
      `${t.preferences}${texts.app.shortcuts.preferences}`,
    );
  });

  it("disables a tool whose function is not mounted, with the reason", async () => {
    const { calls } = renderWithActions(<SidebarTools />, ["preferences"]);
    const validate = screen.getByRole("button", { name: t.validate });

    await userEvent.click(validate);

    expect(validate).toHaveAttribute("aria-disabled", "true");
    expect(validate).toHaveAccessibleDescription(texts.app.reasons.unavailable);
    expect(calls).toEqual([]);
  });
});
