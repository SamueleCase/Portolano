import { describe, expect, it } from "vitest";

import { editInput, newNodeInput } from "./editorRequest";
import { blankValues, type EditorValues } from "./editorValues";
import { reapply } from "./externalChanges";
import { routeError } from "./serverErrors";
import { settingsOf } from "./settingsOf";

const NO_SECRETS = new Map<string, string>();

function ssh(changes: Partial<EditorValues> = {}): EditorValues {
  return { ...blankValues("ssh", null), name: "web09", host: "web09.example.com", ...changes };
}

describe("newNodeInput", () => {
  it("sends no port and no settings for a connection left at its defaults", () => {
    const input = newNodeInput(ssh(), NO_SECRETS);

    expect(input.kind).toEqual({
      type: "connection",
      protocol: "ssh",
      host: "web09.example.com",
      port: null,
      credential: { mode: "inherit" },
      settings: {},
    });
  });

  it("sends a staged secret only with an inline credential", () => {
    const values = ssh({ secrets: { password: "set" } });
    const staged = new Map([["password", "valore-di-prova"]]);

    expect(newNodeInput(values, staged).secrets?.password).toBeNull();
    expect(
      newNodeInput(
        { ...values, credential: { mode: "inline", username: "ops", domain: null } },
        staged,
      ).secrets?.password,
    ).toBe("valore-di-prova");
  });
});

describe("editInput", () => {
  it("sends only the name when only the name changed", () => {
    const base = ssh();

    const edit = editInput({ ...base, name: "web10" }, base, {}, NO_SECRETS);

    expect(edit).toMatchObject({ name: "web10", secrets: { password: null } });
    expect(edit).not.toHaveProperty("host");
    expect(edit).not.toHaveProperty("settings");
    expect(edit).not.toHaveProperty("credential");
  });

  it("puts the port back to the default of the protocol when it is emptied", () => {
    const base = ssh({ port: "2222" });

    expect(editInput({ ...base, port: "" }, base, {}, NO_SECRETS)).toMatchObject({
      defaultPort: true,
    });
  });

  it("sends a staged removal as a change of Salva", () => {
    const base = ssh({ credential: { mode: "inline", username: "ops", domain: null } });

    const edit = editInput({ ...base, secrets: { password: "clear" } }, base, {}, NO_SECRETS);

    expect(edit.secrets?.password).toEqual({ action: "clear" });
  });
});

describe("settingsOf", () => {
  it("writes the ssh settings as the backend stores them", () => {
    const values = ssh({
      jumpHosts: [
        { id: "a", ref: "bastion-id" },
        { id: "b", address: " deploy@198.51.100.40:22 " },
      ],
      agentForwarding: true,
      keepalive: "30",
      options: [
        { id: "o", key: "ConnectTimeout", value: "5" },
        { id: "e", key: "", value: "" },
      ],
    });

    expect(settingsOf(values)).toEqual({
      jump_hosts: [{ $ref: "bastion-id" }, { address: "deploy@198.51.100.40:22" }],
      agent_forwarding: true,
      keepalive_s: 30,
      options: { ConnectTimeout: "5" },
    });
  });

  it("keeps the stored credential of a gateway the editor does not change", () => {
    const stored = { mode: "inline", username: "gw", domain: null, password: "ref-1" };
    const values: EditorValues = {
      ...blankValues("rdp", null),
      gatewayHost: "gw.example.com",
      gatewayCredential: { mode: "kept", stored: "inline", username: "gw" },
      clipboard: false,
    };

    expect(settingsOf(values, { gateway: { host: "old", credential: stored } })).toEqual({
      gateway: { host: "gw.example.com", credential: stored },
      clipboard: false,
    });
  });

  it("keeps the stored settings the editor does not manage", () => {
    expect(settingsOf(ssh(), { future_setting: 1, compression: true })).toEqual({
      future_setting: 1,
    });
  });
});

describe("reapply", () => {
  it("takes the new values of the fields the user did not touch", () => {
    const loaded = ssh();
    const current = { ...loaded, description: "mia" };
    const reloaded = { ...loaded, host: "nuovo.example.com" };

    const result = reapply(loaded, current, reloaded);

    expect(result.kept).toEqual(["description"]);
    expect(result.conflicts).toEqual([]);
  });

  it("reports a field changed differently on both sides", () => {
    const loaded = ssh();

    const result = reapply(
      loaded,
      { ...loaded, host: "mio.example.com" },
      { ...loaded, host: "loro.example.com" },
    );

    expect(result.conflicts).toEqual(["host"]);
  });
});

describe("routeError", () => {
  const invalid = (field: string, message: string) =>
    routeError({ presentation: "field", title: "", message, field }, "ssh");

  it("puts a refused ssh option on its row of the free options", () => {
    expect(invalid("settings.options.ProxyCommand", "non ammessa")).toEqual({
      target: "options",
      message: "non ammessa",
      option: "ProxyCommand",
    });
  });

  it("puts an invalid jump host under the jump hosts", () => {
    expect(invalid("settings.jump_hosts", "non valido").target).toBe("jumpHosts");
  });

  it("keeps the settings as a whole for the bottom of the panel", () => {
    expect(invalid("settings", "non valide").target).toBe("form");
  });

  it("puts a name already taken under the name", () => {
    expect(invalid("name", "esiste già un nodo chiamato 'web01'").target).toBe("name");
  });

  it("keeps an error without a field for the bottom of the panel", () => {
    expect(
      routeError({ presentation: "reloadBanner", title: "", message: "modificato" }, "ssh").target,
    ).toBe("form");
  });
});
