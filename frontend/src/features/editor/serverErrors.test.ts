import { describe, expect, it } from "vitest";

import type { PresentedError } from "../../ipc/errors";
import { routeError, stepOf } from "./serverErrors";

/** An invalid value of the backend on `field`. */
function onField(field: string): PresentedError {
  return {
    presentation: "field",
    title: "Valore non valido",
    message: "valore non valido",
    field,
    details: { kind: "invalid", field },
  };
}

describe("routeError", () => {
  it("puts an error of a secret property under the properties, on the first step", () => {
    const routed = routeError(onField("property:Token API"), "ssh");

    expect(routed.target).toBe("properties");
    expect(stepOf(routed.target)).toBe(0);
  });

  it("puts a refused ssh option on the options, with the option", () => {
    expect(routeError(onField("settings.options.ProxyCommand"), "ssh")).toEqual({
      target: "options",
      message: "valore non valido",
      option: "ProxyCommand",
    });
  });

  it("puts the settings as a whole at the bottom of the panel", () => {
    expect(routeError(onField("settings"), "ssh").target).toBe("form");
  });
});
