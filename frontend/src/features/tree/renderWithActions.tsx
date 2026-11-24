import { render } from "@testing-library/react";
import type { ReactElement } from "react";

import type { Feature } from "../../app/features";
import { type WindowActions, WindowActionsContext } from "../../app/windowActions";

/** All the hook points: the default of {@link renderWithActions}. */
const EVERY_FEATURE: readonly Feature[] = [
  "editor",
  "password",
  "unlockVault",
  "createFileVault",
  "login",
  "import",
  "preferences",
  "validation",
];

/**
 * Renders `ui` with window actions that record their calls, as "name arg…";
 * the functions of `offered` are the mounted ones. For the tests only.
 */
export function renderWithActions(ui: ReactElement, offered: readonly Feature[] = EVERY_FEATURE) {
  const calls: string[] = [];
  const record =
    (name: string) =>
    (...args: unknown[]) =>
      calls.push([name, ...args.map(String)].join(" "));
  // Every action records its name: the proxy stands for the whole interface.
  const actions = new Proxy({} as WindowActions, {
    get: (_target, name: string) =>
      name === "offers" ? (feature: Feature) => offered.includes(feature) : record(name),
  });
  const view = render(<WindowActionsContext value={actions}>{ui}</WindowActionsContext>);
  return { calls, ...view };
}
