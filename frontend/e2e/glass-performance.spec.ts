import { expect, test } from "./fixtures";

/**
 * The glass measurement of DEC-028.6, run by `npm run perf:glass` and not by
 * the normal suite: the animated background under three glass panels at
 * 1280×800 with DPR 2. Times depend on the machine and are only printed; the
 * test fails only if the background draws more than 30 times per second or
 * keeps drawing while the page is hidden.
 */

declare global {
  interface Window {
    __draws: number;
  }
}

const SECONDS = 10;
const MAX_DRAWS_PER_SECOND = 30;

test.use({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 2 });

function percentile(sorted: number[], share: number): number {
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * share))] ?? 0;
}

test("the glass background stays within 30 drawings per second and stops when hidden", async ({
  page,
}) => {
  test.setTimeout(120_000);
  // Every drawing of the constellation starts by clearing the canvas.
  await page.addInitScript(() => {
    window.__draws = 0;
    const prototype = CanvasRenderingContext2D.prototype;
    // eslint-disable-next-line @typescript-eslint/unbound-method -- called below with its own receiver
    const clear = prototype.clearRect;
    prototype.clearRect = function (this: CanvasRenderingContext2D, ...area) {
      window.__draws += 1;
      clear.apply(this, area);
    };
  });
  await page.goto("/?gallery=glass");
  await page.waitForFunction(() => window.__draws > 0);

  const sample = await page.evaluate(async (seconds) => {
    const frames: number[] = [];
    const drawsBefore = window.__draws;
    const start = performance.now();
    await new Promise<void>((done) => {
      let last = start;
      const frame = (now: number) => {
        frames.push(now - last);
        last = now;
        if (now - start < seconds * 1000) requestAnimationFrame(frame);
        else done();
      };
      requestAnimationFrame(frame);
    });
    const elapsed = (performance.now() - start) / 1000;
    return { frames, draws: window.__draws - drawsBefore, elapsed, focused: document.hasFocus() };
  }, SECONDS);

  const sorted = [...sample.frames].sort((a, b) => a - b);
  const drawsPerSecond = sample.draws / sample.elapsed;
  console.log(
    [
      `glass-performance: ${sample.frames.length} frame in ${sample.elapsed.toFixed(1)} s`,
      `frame p50 ${percentile(sorted, 0.5).toFixed(1)} ms, p95 ${percentile(sorted, 0.95).toFixed(1)} ms`,
      `disegni ${sample.draws} (${drawsPerSecond.toFixed(1)}/s), finestra a fuoco: ${String(sample.focused)}`,
    ].join("\n"),
  );
  // Without focus or drawings there would be nothing measured.
  expect(sample.focused, "la finestra della misura deve avere il fuoco").toBe(true);
  expect(sample.draws, "lo sfondo deve disegnare durante la misura").toBeGreaterThan(0);
  expect(drawsPerSecond).toBeLessThanOrEqual(MAX_DRAWS_PER_SECOND);

  // A hidden page: the visibility the WebView reports when the window is hidden.
  const hiddenDraws = await page.evaluate(async () => {
    Object.defineProperty(document, "hidden", { configurable: true, get: () => true });
    Object.defineProperty(document, "visibilityState", { configurable: true, get: () => "hidden" });
    document.dispatchEvent(new Event("visibilitychange"));
    const before = window.__draws;
    await new Promise((done) => setTimeout(done, 2000));
    return window.__draws - before;
  });
  console.log(`glass-performance: disegni con la pagina nascosta: ${hiddenDraws}`);
  expect(hiddenDraws).toBe(0);
});
