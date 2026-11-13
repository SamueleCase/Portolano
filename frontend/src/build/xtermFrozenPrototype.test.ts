import { runInNewContext } from "node:vm";

import { describe, expect, it } from "vitest";

import { defineXtermToString, xtermModuleId } from "./xtermFrozenPrototype";

/** The statement of xterm 6.0.0, as minified in `lib/xterm.mjs`. */
const minified =
  "(o=>{function s(l){return l}o.toString=s;function t(l){return l}o.fromString=t})(Qn||={});";

const definition =
  'Object.defineProperty(o, "toString", { value: s, writable: true, configurable: true, enumerable: true });';

describe("defineXtermToString", () => {
  it("rewrites the toString assignment as a property definition", () => {
    expect(defineXtermToString(minified)).toBe(
      `(o=>{function s(l){return l}${definition}function t(l){return l}o.fromString=t})(Qn||={});`,
    );
  });

  it("rewrites the assignment whatever its spacing", () => {
    expect(defineXtermToString("function s() {}\no.toString = s;\n")).toBe(
      `function s() {}\n${definition}\n`,
    );
  });

  it("produces code that runs where the assignment throws on a frozen prototype", () => {
    // A frozen prototype of its own stands for the frozen Object.prototype,
    // which the test runner itself needs writable.
    const run = (code: string) => {
      const prototype = Object.freeze({ toString: () => "prototype" });
      const namespace = Object.create(prototype) as typeof prototype;
      runInNewContext(`"use strict"; ${code}`, { o: namespace });
      return namespace.toString();
    };
    const original = 'function s() { return "Alt"; }\no.toString = s;';
    // The TypeError of the context is not the TypeError of the test.
    expect(() => run(original)).toThrow(/read only/);
    expect(run(defineXtermToString(original))).toBe("Alt");
  });

  it("refuses a module without the assignment", () => {
    expect(() => defineXtermToString("o.fromString=t;")).toThrow(/trovate 0/);
  });

  it("refuses a module with the assignment twice", () => {
    expect(() => defineXtermToString("o.toString=s;a.toString=b;")).toThrow(/trovate 2/);
  });

  it("does not take a member of a member for the assignment", () => {
    expect(() => defineXtermToString("a.o.toString=s;")).toThrow(/trovate 0/);
  });
});

describe("xtermModuleId", () => {
  it("matches the module of xterm", () => {
    expect(xtermModuleId.test("/app/node_modules/@xterm/xterm/lib/xterm.mjs")).toBe(true);
  });

  it("leaves the addons of xterm alone", () => {
    expect(xtermModuleId.test("/app/node_modules/@xterm/addon-fit/lib/addon-fit.mjs")).toBe(false);
  });
});
