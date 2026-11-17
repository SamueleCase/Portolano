import { screen } from "@testing-library/react";
import type { ComponentType } from "react";
import { describe, expect, it } from "vitest";

import { texts } from "../../i18n/it";
import { installFakeBackend } from "../../test/fakeBackend";
import { renderWithQueries } from "../../test/render";
import { ChoiceSetting } from "../preferences/ChoiceSetting";
import { CredentialStep } from "./CredentialStep";
import { type EditorFields, EditorFieldsContext } from "./editorContext";
import { blankValues, type EditorKind } from "./editorValues";
import { GeneralStep } from "./GeneralStep";
import { secretSlots } from "./secretSlots";
import type { ErrorTarget } from "./serverErrors";
import { SshStep } from "./SshStep";

const t = texts.editor;
const MESSAGE = "valore rifiutato dal backend";

/** `Step` of a `kind` node whose only error is `MESSAGE` on `target`. */
function renderStep(Step: ComponentType, kind: EditorKind, target: ErrorTarget) {
  installFakeBackend();
  const fields: EditorFields = {
    values: { ...blankValues(kind, null), keyMode: "vault" },
    set: () => undefined,
    errorOf: (asked) => (asked === target ? MESSAGE : undefined),
    refusedOption: undefined,
    secrets: secretSlots([], [], {}),
    askSecret: () => undefined,
    clearSecret: () => undefined,
    backend: "keychain",
    nodeId: undefined,
    newRowId: () => "new:1",
  };
  renderWithQueries(
    <EditorFieldsContext.Provider value={fields}>
      <Step />
    </EditorFieldsContext.Provider>,
  );
}

/** The group the error sits in. */
function groupOfError(): HTMLElement {
  const group = screen.getByText(MESSAGE).closest<HTMLElement>("[role='group']");
  if (!group) throw new Error("errore fuori da un gruppo");
  return group;
}

describe("the errors of a group of controls", () => {
  it("describe the button that chooses the icon", () => {
    renderStep(GeneralStep, "ssh", "icon");

    expect(screen.getByRole("button", { name: t.general.chooseIcon })).toHaveAccessibleDescription(
      MESSAGE,
    );
  });

  it("describe the group of the color", () => {
    renderStep(GeneralStep, "ssh", "color");

    expect(groupOfError()).toHaveAccessibleDescription(MESSAGE);
  });

  it("describe the button that changes the position", () => {
    renderStep(GeneralStep, "ssh", "parent");

    expect(screen.getByRole("button", { name: t.general.change })).toHaveAccessibleDescription(
      MESSAGE,
    );
  });

  it("describe the properties", () => {
    renderStep(GeneralStep, "ssh", "properties");

    expect(screen.getByRole("group", { name: t.general.properties })).toHaveAccessibleDescription(
      MESSAGE,
    );
  });

  it("describe the jump hosts", () => {
    renderStep(SshStep, "ssh", "jumpHosts");

    expect(screen.getByRole("group", { name: t.ssh.jumps })).toHaveAccessibleDescription(MESSAGE);
  });

  it("describe the ssh options when no row has the error", () => {
    renderStep(SshStep, "ssh", "options");

    expect(screen.getByRole("group", { name: t.ssh.options })).toHaveAccessibleDescription(MESSAGE);
  });

  it("describe the credential", () => {
    renderStep(SshStep, "ssh", "credential");

    expect(groupOfError()).toHaveAccessibleDescription(MESSAGE);
  });

  it("describe a secret of a credential", () => {
    renderStep(CredentialStep, "credential", "privateKey");

    expect(groupOfError()).toHaveAccessibleDescription(MESSAGE);
  });

  it("describe a choice of the preferences", () => {
    renderWithQueries(
      <ChoiceSetting
        label={texts.preferences.vault.label}
        options={[{ value: "keychain", label: "Portachiavi" }]}
        value="keychain"
        onChange={() => undefined}
        error={MESSAGE}
      />,
    );

    expect(groupOfError()).toHaveAccessibleDescription(MESSAGE);
  });
});
