import { app } from "./app";
import { components } from "./components";
import { details } from "./details";
import { dialogs } from "./dialogs";
import { editor } from "./editor";
import { firstRun } from "./firstRun";
import { icons } from "./icons";
import { importWizard } from "./import";
import { ipc } from "./ipc";
import { preferences } from "./preferences";
import { sessions } from "./sessions";
import { tree } from "./tree";
import { validation } from "./validation";

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
  dialogs,
  editor,
  preferences,
  importWizard,
  validation,
} as const;
