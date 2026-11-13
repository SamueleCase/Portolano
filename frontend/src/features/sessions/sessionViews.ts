import { ExternalSessionView } from "./ExternalSessionView";
import { RdpSessionView } from "./RdpSessionView";
import type { SessionViews } from "./SessionView";
import { TerminalSessionView } from "./TerminalSessionView";

/**
 * The views of the app, by kind of session (SPEC-UI §5.1): the integrated RDP
 * panel will add its own here.
 */
export const sessionViews: SessionViews = {
  terminal: TerminalSessionView,
  "rdp-external": RdpSessionView,
  external: ExternalSessionView,
};
