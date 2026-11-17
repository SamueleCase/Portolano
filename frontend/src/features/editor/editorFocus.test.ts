import { afterEach, describe, expect, it } from "vitest";

import { controlWithError, firstFieldOfStep } from "./editorFocus";

/** A form of the editor with `html` as its step. */
function form(html: string): HTMLFormElement {
  const element = document.createElement("form");
  element.innerHTML = `<section>${html}</section><footer><button>Salva</button></footer>`;
  document.body.append(element);
  return element;
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("firstFieldOfStep", () => {
  it("is the first field of the step, past its buttons", () => {
    const step = form(`<button>Cambia</button><input aria-label="Host">`);

    expect(firstFieldOfStep(step)?.getAttribute("aria-label")).toBe("Host");
  });

  it("is the first control of a step without fields", () => {
    const step = form(`<button>Cambia</button>`);

    expect(firstFieldOfStep(step)?.textContent).toBe("Cambia");
  });
});

describe("controlWithError", () => {
  it("is a field marked invalid", () => {
    const step = form(`<input aria-label="Nome"><input aria-label="Host" aria-invalid="true">`);

    expect(controlWithError(step)?.getAttribute("aria-label")).toBe("Host");
  });

  it("is a control described by an error, not by a hint", () => {
    const step = form(`
      <input aria-label="Porta" aria-describedby="hint"><span id="hint">predefinita</span>
      <button aria-describedby="error">Cambia</button><span id="error" role="alert">no</span>`);

    expect(controlWithError(step)?.textContent).toBe("Cambia");
  });

  it("is the option holding the tab stop of a group of radios described by an error", () => {
    const step = form(`
      <div role="radiogroup">
        <button role="radio" tabindex="-1" aria-describedby="error">Nessuno</button>
        <button role="radio" tabindex="0" aria-describedby="error">Blu</button>
      </div><span id="error" role="alert">no</span>`);

    expect(controlWithError(step)?.textContent).toBe("Blu");
  });

  it("is the first control of a named group described by an error", () => {
    const step = form(`
      <div role="group" aria-label="Jump host" aria-describedby="error">
        <button>Dall'albero</button>
      </div><span id="error" role="alert">no</span>`);

    expect(controlWithError(step)?.textContent).toBe("Dall'albero");
  });

  it("is none without an error", () => {
    expect(controlWithError(form(`<input aria-label="Nome">`))).toBeUndefined();
  });
});
