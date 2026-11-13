import { test as base, expect } from "@playwright/test";

/** A violation of the CSP of the app, as the browser reports it. */
interface CspViolation {
  directive: string;
  blocked: string;
}

declare global {
  interface Window {
    __cspViolations: CspViolation[];
  }
}

/**
 * The test of every end-to-end scenario. The page runs as in Tauri: with
 * `Object.prototype` frozen before any script of its own, and served with the
 * CSP of the app (`vite.config.ts`); a scenario fails if anything on the page
 * needed more than the policy allows.
 */
export const test = base.extend<{ frozenPrototype: undefined; cspGuard: undefined }>({
  // `freezePrototype` of `tauri.conf.json` (SPEC-UI §4.8): a library that
  // assigns over a property of `Object.prototype` throws in the app, so it
  // must throw here too, and the scenario fails on the error even when it
  // leaves the screen apparently whole (DEC-034).
  frozenPrototype: [
    async ({ page }, use) => {
      const pageErrors: string[] = [];
      page.on("pageerror", (error) => pageErrors.push(String(error)));
      await page.addInitScript(() => {
        Object.freeze(Object.prototype);
      });
      await use(undefined);
      expect(pageErrors, "errori non gestiti della pagina").toEqual([]);
    },
    { auto: true },
  ],
  cspGuard: [
    async ({ page }, use) => {
      await page.addInitScript(() => {
        window.__cspViolations = [];
        document.addEventListener("securitypolicyviolation", (event) => {
          window.__cspViolations.push({
            directive: event.violatedDirective,
            blocked: event.blockedURI,
          });
        });
      });
      await use(undefined);
      const violations = await page.evaluate(() => window.__cspViolations);
      expect(violations, "violazioni della CSP").toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };
