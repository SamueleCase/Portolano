import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { SecretRequestView_Serialize } from "../../bindings";
import { ImportAnswers, useImportAnswers } from "./importAnswers";

const SENTINEL = "sentinel-answers-secret-0c3d";

const REQUESTS: SecretRequestView_Serialize[] = [
  { nodePath: "Ufficio/web10", username: "deploy", field: { kind: "password" } },
  { nodePath: "Ufficio/db10", username: null, field: { kind: "passphrase" } },
];

describe("ImportAnswers", () => {
  it("gives the answers in the order of the requests, the missing ones skipped", () => {
    const answers = new ImportAnswers();
    answers.start(REQUESTS);
    answers.record(SENTINEL);

    expect(answers.take(REQUESTS)).toEqual([
      { nodePath: "Ufficio/web10", field: { kind: "password" }, value: SENTINEL },
      { nodePath: "Ufficio/db10", field: { kind: "passphrase" }, value: null },
    ]);
  });

  it("lets the passwords go once taken", () => {
    const answers = new ImportAnswers();
    answers.start(REQUESTS);
    answers.record(SENTINEL);

    answers.take(REQUESTS);

    expect(answers.holdsPasswords()).toBe(false);
  });

  it("skips every request of a preview it did not start with", () => {
    const answers = new ImportAnswers();
    answers.start(REQUESTS);
    answers.record(SENTINEL);

    const taken = answers.take([...REQUESTS]);

    expect(taken.map((answer) => answer.value)).toEqual([null, null]);
  });

  it("ignores answers beyond the last request", () => {
    const answers = new ImportAnswers();
    answers.start(REQUESTS);

    ["uno", "due", "tre"].forEach((value) => answers.record(value));

    expect(answers.position).toBe(2);
  });

  it("drops the passwords when the wizard goes away", () => {
    const { result, unmount } = renderHook(() => useImportAnswers());
    result.current.current.start(REQUESTS);
    result.current.current.record(SENTINEL);

    unmount();

    expect(result.current.current.holdsPasswords()).toBe(false);
  });
});
