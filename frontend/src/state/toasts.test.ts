import { beforeEach, describe, expect, it } from "vitest";

import { showToast, useToasts } from "./toasts";

describe("the toasts", () => {
  beforeEach(() => useToasts.setState({ toasts: [] }));

  it("gives every toast its own id, in order", () => {
    const first = showToast({ kind: "info", text: "uno" });
    const second = showToast({ kind: "error", text: "due" });

    expect(first).not.toBe(second);
    expect(useToasts.getState().toasts.map((toast) => toast.text)).toEqual(["uno", "due"]);
  });

  it("forgets a dismissed toast", () => {
    const id = showToast({ kind: "info", text: "uno" });

    useToasts.getState().dismiss(id);

    expect(useToasts.getState().toasts).toEqual([]);
  });
});
