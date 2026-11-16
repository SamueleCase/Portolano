import { app } from "./app";
import { components } from "./components";
import { details } from "./details";
import { firstRun } from "./firstRun";
import { icons } from "./icons";
import { ipc } from "./ipc";
import { sessions } from "./sessions";
import { tree } from "./tree";

/**
 * The texts of the interface, in Italian (SPEC-UI: no text outside i18n/).
 * One file per area, one key per area here (DEC-029.5): a work package adds
 * its file and its line.
 */
export const texts = {
  components,
  icons,
  ipc,
  sessions,
  app,
  tree,
  details,
  firstRun,
} as const;
