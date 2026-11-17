import { describe, expect, it } from "vitest";

import { readableImportPath } from "./importPath";

describe("readableImportPath", () => {
  it("separates the names as the glossary does", () => {
    expect(readableImportPath("Ufficio/Rete/web01")).toBe("Ufficio / Rete / web01");
  });

  it("keeps a slash that is part of a name", () => {
    expect(readableImportPath("Ufficio/LAN\\/WAN/router")).toBe("Ufficio / LAN/WAN / router");
  });

  it("keeps a backslash that is part of a name", () => {
    expect(readableImportPath("Ufficio/DOMINIO\\\\admin")).toBe("Ufficio / DOMINIO\\admin");
  });
});
