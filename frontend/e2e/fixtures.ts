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
 * The test of every end-to-end scenario: the page is served with the CSP of
 * the app (`vite.config.ts`), and a scenario fails if anything on the page
 * needed more than the policy allows.
 */
export const test = base.extend<{ cspGuard: undefined }>({
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
