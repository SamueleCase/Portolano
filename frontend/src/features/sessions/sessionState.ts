import { texts } from "../../i18n/it";
import type { SessionKind, SessionState } from "../../ipc/sessions";

/** The phase of a session, which colors its dot and picks its actions. */
export type SessionPhase = SessionState["state"];

/** Whether the program of the session is over. */
export function isOver(state: SessionState): boolean {
  return state.state === "ended" || state.state === "failed";
}

/** Whether "Riprova" makes sense: the connection did not succeed. */
export function canRetry(state: SessionState): boolean {
  return state.state === "failed";
}

/** The exit code of a program that is over, if it has one. */
export function exitCodeOf(state: SessionState): number | undefined {
  switch (state.state) {
    case "ended":
      return state.code;
    case "failed":
      return state.code ?? undefined;
    default:
      return undefined;
  }
}

/**
 * The status shown in the tab: one state for the whole connection (ambiguity
 * 11); "connected" reads differently when the connection lives in another
 * window (ambiguity 12).
 */
export function statusLabel(kind: SessionKind, state: SessionState): string {
  const status = texts.sessions.status;
  switch (state.state) {
    case "connecting":
      return status.connecting;
    case "connected":
      if (kind === "rdp-external") {
        return status.rdpRunning;
      }
      return kind === "external" ? status.externalOpen : status.connected;
    case "ended":
      return status.endedWithCode(state.code);
    case "failed":
      return state.code === null ? status.failed : status.failedWithCode(state.code);
  }
}

/** A duration as `hh:mm:ss`, as the RDP tab shows it. */
export function formatDuration(milliseconds: number): string {
  const seconds = Math.max(0, Math.floor(milliseconds / 1000));
  return [Math.floor(seconds / 3600), Math.floor(seconds / 60) % 60, seconds % 60]
    .map((part) => String(part).padStart(2, "0"))
    .join(":");
}
