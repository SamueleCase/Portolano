import { expect, test } from "./fixtures";
import { openGallery } from "./galleryPage";
import { markControls, missedControls, sweep, TAB, withoutTransitions } from "./keyboardPage";

test("every control of the gallery is reached with Tab and shows its focus", async ({ page }) => {
  test.slow();
  await openGallery(page);
  await withoutTransitions(page);
  const total = await markControls(page);

  const stops = await sweep(page, TAB, total + 20);

  expect(await missedControls(page, total, stops), "controlli non raggiungibili con Tab").toEqual(
    [],
  );
  expect(
    stops.filter((stop) => !stop.visible).map((stop) => stop.name),
    "controlli senza fuoco visibile",
  ).toEqual([]);
  expect(
    stops.some((stop) => stop.terminal),
    "il terminale si raggiunge e si lascia",
  ).toBe(true);
});
