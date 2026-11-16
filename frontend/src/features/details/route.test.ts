import { describe, expect, it } from "vitest";

import { commands } from "../../bindings";
import { installFakeBackend } from "../../test/fakeBackend";
import { idOf, ok } from "../../test/fakeBackendFixtures";
import { routeOf } from "./route";

async function detailOf(name: string) {
  installFakeBackend();
  return ok(await commands.nodeDetails(await idOf(name)));
}

describe("the route of a connection", () => {
  it("goes through the hosts of its jumps to its own host", async () => {
    expect(routeOf(await detailOf("web01"))).toEqual(["bastion.example.com", "web01.example.com"]);
  });

  it("has no route without jumps", async () => {
    expect(routeOf(await detailOf("bastion"))).toBeUndefined();
  });

  it("shows a jump to a node that is gone as a dash", async () => {
    const detail = await detailOf("web01");
    const connection = detail.connection;
    if (!connection) throw new Error("web01 è una connessione");

    expect(routeOf({ ...detail, connection: { ...connection, jumps: [null] } })).toEqual([
      "—",
      "web01.example.com",
    ]);
  });
});
