import { describe, expect, it } from "vitest";

import { BUILTIN_ICONS } from "../bindings";
import { builtinIconNames } from "../components/icons/builtinIconNames";
import { builtinIcons } from "../components/icons/builtinIcons";

/** DEC-029.4: the names of the backend and the icons of the interface are the same set. */
describe("builtin icons of the backend", () => {
  it("each have an icon in the interface, and no icon has another name", () => {
    expect(Object.keys(builtinIcons).sort()).toEqual([...BUILTIN_ICONS].sort());
  });

  it("are the names the interface knows, in the same order", () => {
    expect([...builtinIconNames]).toEqual([...BUILTIN_ICONS]);
  });
});
