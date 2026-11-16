import { describe, expect, it } from "vitest";

import type { Issue_Serialize } from "../../bindings";
import { validationTabLabel } from "./validationSummary";

function issues(count: number): Issue_Serialize[] {
  return Array.from({ length: count }, (_, index) => ({
    severity: "warning",
    node: `node-${index}`,
    code: "missing_secret",
    message: "un segreto del nodo non è nel vault",
  }));
}

describe("validationTabLabel", () => {
  it("names a single problem in the singular", () => {
    expect(validationTabLabel(issues(1))).toBe("1 problema");
  });

  it("counts the problems in the plural", () => {
    expect(validationTabLabel(issues(4))).toBe("4 problemi");
  });

  it("counts no problem in the plural", () => {
    expect(validationTabLabel([])).toBe("0 problemi");
  });
});
