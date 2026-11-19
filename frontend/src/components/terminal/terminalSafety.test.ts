import { describe, expect, it } from "vitest";

import { linkParts, openableLink, shortenMiddle, terminalTitle } from "./terminalSafety";

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

describe("linkParts", () => {
  it("shows the host the link goes to, not the user before the @", () => {
    const parts = linkParts("https://intranet.example.com@evil.example/");

    expect(parts).toEqual({
      host: "evil.example",
      address: "https://evil.example/",
      user: "intranet.example.com",
    });
  });

  it("keeps the whole host and shortens a long path in the middle", () => {
    const parts = linkParts(`https://docs.example.com:8443/${"a".repeat(200)}/fine`);

    expect(parts.host).toBe("docs.example.com:8443");
    expect(parts.address).toMatch(/^https:\/\/docs\.example\.com:8443\/a+…a*\/fine$/);
    expect([...parts.address].length).toBeLessThan(100);
  });

  it("says there are no credentials when the link has none", () => {
    expect(linkParts("https://www.example.com/docs").user).toBeNull();
  });
});

describe("shortenMiddle", () => {
  it("keeps a short text as it is", () => {
    expect(shortenMiddle("/docs", 10)).toBe("/docs");
  });

  it("keeps the start and the end of a long text, at most the length given", () => {
    expect(shortenMiddle("/0123456789", 7)).toBe("/01…789");
  });
});
