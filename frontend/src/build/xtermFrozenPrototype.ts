import type { Plugin } from "vite";

// Tauri freezes `Object.prototype` before the page (`freezePrototype`,
// SPEC-UI §4.8). `@xterm/xterm` 6.0.0 runs `o.toString = s` on a plain object
// when its module loads: with the prototype frozen the inherited `toString` is
// read-only, so the assignment throws ("override mistake") and every screen
// that imports the terminal goes blank. `Object.defineProperty` creates the
// same own property without consulting the prototype (DEC-034).

/** The module of `@xterm/xterm` that the bundlers load (`module` of its package). */
export const xtermModuleId = /[\\/]node_modules[\\/]@xterm[\\/]xterm[\\/]lib[\\/]xterm\.mjs$/;

/** `<object>.toString = <function>;` as a whole statement, whatever the spacing. */
const toStringAssignment = /(?<![\w$.])([A-Za-z_$][\w$]*)\.toString\s*=\s*([A-Za-z_$][\w$]*)\s*;/g;

/** The assignments that xterm 6.0.0 contains; any other count means a different xterm. */
const expectedAssignments = 1;

/**
 * Rewrites the `toString` assignment of xterm as a property definition that
 * works with a frozen `Object.prototype`. Throws if the assignment does not
 * appear exactly once, so an update of xterm is reviewed instead of breaking
 * the app (or being patched in the wrong place) in silence.
 */
export function defineXtermToString(code: string): string {
  const found = code.match(toStringAssignment)?.length ?? 0;
  if (found !== expectedAssignments) {
    throw new Error(
      `@xterm/xterm: attese ${expectedAssignments} assegnazioni "<oggetto>.toString = <funzione>" ` +
        `da riscrivere per freezePrototype, trovate ${found}. Verificare la nuova versione di xterm ` +
        `con Object.prototype congelato e aggiornare src/build/xtermFrozenPrototype.ts (DEC-034).`,
    );
  }
  return code.replace(
    toStringAssignment,
    'Object.defineProperty($1, "toString", { value: $2, writable: true, configurable: true, enumerable: true });',
  );
}

/**
 * The plugin that applies `defineXtermToString` to the module of xterm. It is
 * a plain Rolldown plugin, so the same object serves the build and the
 * pre-bundling of the dependencies in development (`optimizeDeps`).
 */
export function xtermFrozenPrototype(): Plugin {
  return {
    name: "portolano:xterm-frozen-prototype",
    transform: {
      filter: { id: xtermModuleId },
      handler(code) {
        // The rewrite stays on one line of a minified file: a null map keeps
        // the original lines, and only the columns after it drift.
        return { code: defineXtermToString(code), map: null };
      },
    },
  };
}
