import { describe, expect, it } from "vitest";

import { icons } from "../../i18n/icons";
import { builtinIconNames, isBuiltinIconName } from "./builtinIconNames";
import { builtinIcons } from "./builtinIcons";

// DEC-029.4: exactly these names, in this order.
const decided = [
  "server",
  "desktop",
  "laptop",
  "database",
  "terminal",
  "cloud",
  "network",
  "router",
  "globe",
  "key",
  "lock",
  "shield",
  "folder",
  "hard-drive",
  "cpu",
  "container",
  "box",
  "building",
  "home",
  "user",
  "users",
  "mail",
  "printer",
  "phone",
  "wifi",
  "git-branch",
  "code",
  "settings",
  "star",
  "tag",
];

describe("builtin icons", () => {
  it("are exactly the names of DEC-029.4", () => {
    expect([...builtinIconNames]).toEqual(decided);
  });

  it("map every name and only those to a Lucide icon", () => {
    expect(Object.keys(builtinIcons).sort()).toEqual([...decided].sort());
  });

  it("have an Italian label for every name and only those", () => {
    expect(Object.keys(icons).sort()).toEqual([...decided].sort());
  });

  it("recognise a builtin name read from the document", () => {
    expect(isBuiltinIconName("hard-drive")).toBe(true);
  });

  it("reject a name outside the set", () => {
    expect(isBuiltinIconName("skull")).toBe(false);
  });
});
