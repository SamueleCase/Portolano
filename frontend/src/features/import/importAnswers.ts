import { type RefObject, useEffect, useRef } from "react";

import type { ImportAnswerInput_Deserialize, SecretRequestView_Serialize } from "../../bindings";

/**
 * The passwords typed in step 3 of the wizard, until `import_apply`.
 *
 * SPEC-UI §1.2 wants a typed secret to reach the backend with a single
 * command and to live nowhere else. `import_apply` takes every answer at
 * once, in the order of the requests, so the answers typed one at a time
 * have to wait for the last one somewhere: here, in plain memory held by a
 * ref of the wizard. Never in React state (renders, devtools), in Zustand or
 * in the cache of TanStack Query (`useSecretMutation` keeps them out of it
 * too), never logged. The field of each password is emptied as soon as its
 * value moves here, and the values leave on every path: `take()` before the
 * import (successful or not), `clear()` when the user goes back or the
 * wizard closes. JavaScript strings cannot be zeroed: a limit of web UIs
 * documented in the README (SPEC-UI §1.2).
 */
export class ImportAnswers {
  #requests: readonly SecretRequestView_Serialize[] = [];
  #values: (string | null)[] = [];

  /** Starts over with the requests of the preview. */
  start(requests: readonly SecretRequestView_Serialize[]): void {
    this.#requests = requests;
    this.#values = [];
  }

  /** The answer to the current request: a password, or null for "Salta". */
  record(value: string | null): void {
    if (this.#values.length < this.#requests.length) this.#values.push(value);
  }

  /** The index of the request to answer next. */
  get position(): number {
    return this.#values.length;
  }

  /** Whether every request has its answer. */
  get complete(): boolean {
    return this.#values.length >= this.#requests.length;
  }

  /**
   * The answers to `requests`, in their order, the ones not given yet skipped
   * ("Salta tutte le restanti"); the values leave this object.
   */
  take(requests: readonly SecretRequestView_Serialize[]): ImportAnswerInput_Deserialize[] {
    const values = this.#requests === requests ? this.#values : [];
    const answers = requests.map((request, index) => ({
      nodePath: request.nodePath,
      field: request.field,
      value: values[index] ?? null,
    }));
    this.clear();
    return answers;
  }

  /** Drops the answers given so far. */
  clear(): void {
    this.#values = [];
  }

  /** Whether a typed password is still held: never once the wizard is done with it. */
  holdsPasswords(): boolean {
    return this.#values.some((value) => value !== null);
  }
}

/**
 * The answers of a wizard, in a ref: read and changed only by its event
 * handlers, dropped when the wizard goes away.
 */
export function useImportAnswers(): RefObject<ImportAnswers> {
  const answers = useRef(new ImportAnswers());
  useEffect(() => {
    const held = answers.current;
    return () => held.clear();
  }, []);
  return answers;
}
