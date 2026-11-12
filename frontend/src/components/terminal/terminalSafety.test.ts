import { describe, expect, it } from "vitest";

import { openableLink, terminalTitle } from "./terminalSafety";

describe("openableLink", () => {
  it("opens an https link", () => {
    expect(openableLink("https://www.example.com/docs")).toBe("https://www.example.com/docs");
  });

  it("opens an http link", () => {
    expect(openableLink("http://192.0.2.10:8080/")).toBe("http://192.0.2.10:8080/");
  });

  it("refuses a file link", () => {
    expect(openableLink("file:///etc/passwd")).toBeUndefined();
  });

  it("refuses a javascript link", () => {
    expect(openableLink("javascript:alert(1)")).toBeUndefined();
  });

  it("refuses text that is not a URL", () => {
    expect(openableLink("not a link")).toBeUndefined();
  });
});

describe("terminalTitle", () => {
  it("shows the control characters of a title visibly", () => {
    expect(terminalTitle("web01\u001b[31m")).toBe("web01\\u{1b}[31m");
  });
});
