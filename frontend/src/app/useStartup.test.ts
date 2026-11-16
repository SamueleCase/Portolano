import { describe, expect, it } from "vitest";

import type { AppStatus_Serialize } from "../bindings";
import { initialFakeState } from "../test/fakeBackend";
import { startupSources } from "./useStartup";

const LAST = "/Users/utente-di-prova/infra/portolano.json";
const DEFAULT = "/Users/utente-di-prova/Library/portolano.json";

function status(last: boolean | null, defaultExists: boolean): AppStatus_Serialize {
  return {
    defaultDocument: { path: DEFAULT, exists: defaultExists },
    vault: "file",
    document: null,
    lastDocument: last === null ? null : { path: LAST, exists: last },
    preferences: initialFakeState().preferences,
  };
}

describe("the documents tried at startup", () => {
  it("tries the document opened last, then the default one", () => {
    expect(startupSources(status(true, true))).toEqual([
      { kind: "recent", path: LAST },
      { kind: "default" },
    ]);
  });

  it("falls back to the default document when the last one is gone", () => {
    expect(startupSources(status(false, true))).toEqual([{ kind: "default" }]);
  });

  it("tries nothing without either, for the first run", () => {
    expect(startupSources(status(null, false))).toEqual([]);
  });

  it("does not try the default document twice when it is the last one", () => {
    const same = { ...status(true, true), defaultDocument: { path: LAST, exists: true } };

    expect(startupSources(same)).toEqual([{ kind: "recent", path: LAST }]);
  });
});
