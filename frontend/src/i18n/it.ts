import { components } from "./components";
import { icons } from "./icons";
import { status } from "./status";

/**
 * The texts of the interface, in Italian (SPEC-UI: no text outside i18n/).
 * One file per area, one key per area here (DEC-029.5): a work package adds
 * its file and its line.
 */
export const texts = {
  status,
  components,
  icons,
} as const;
