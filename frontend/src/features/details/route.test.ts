import { describe, expect, it } from "vitest";

import { addressHost, jumpsOf } from "./route";

describe("the route of a connection", () => {
  it("reads the jump hosts of the settings, by reference or by address", () => {
    expect(
      jumpsOf({
        jump_hosts: [
          { $ref: "00000000-0000-4000-8000-000000000103" },
          { address: "u@198.51.100.40:22" },
        ],
      }),
    ).toEqual([{ ref: "00000000-0000-4000-8000-000000000103" }, { address: "u@198.51.100.40:22" }]);
  });

  it("ignores settings without jump hosts or with values it does not know", () => {
    expect(jumpsOf({})).toEqual([]);
    expect(jumpsOf({ jump_hosts: [42, null, { other: "x" }] })).toEqual([]);
  });

  it("keeps only the host of an address", () => {
    expect(addressHost("deploy@198.51.100.40:2222")).toBe("198.51.100.40");
    expect(addressHost("bastion.example.com")).toBe("bastion.example.com");
    expect(addressHost("EXAMPLE\\op@[2001:db8::1]:22")).toBe("2001:db8::1");
  });
});
