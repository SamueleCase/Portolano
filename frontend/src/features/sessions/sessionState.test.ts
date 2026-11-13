import { describe, expect, it } from "vitest";

import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import {
  canRetry,
  exitCodeOf,
  formatDuration,
  isOver,
  sessionDot,
  statusLabel,
} from "./sessionState";

const status = texts.sessions.status;

describe("session states", () => {
  it("are over once ended or failed", () => {
    expect(isOver({ state: "connected" })).toBe(false);
    expect(isOver({ state: "ended", code: 0 })).toBe(true);
    expect(isOver({ state: "failed", code: 255, reason: "rifiutata" })).toBe(true);
  });

  it("offer a retry only when the connection did not succeed", () => {
    expect(canRetry({ state: "failed", code: 255, reason: "rifiutata" })).toBe(true);
    expect(canRetry({ state: "ended", code: 1 })).toBe(false);
  });

  it("carry the exit code of the program once over", () => {
    expect(exitCodeOf({ state: "ended", code: 130 })).toBe(130);
    expect(exitCodeOf({ state: "failed", code: null, reason: "persa" })).toBeUndefined();
    expect(exitCodeOf({ state: "connecting" })).toBeUndefined();
  });
});

describe("statusLabel", () => {
  it("is one state for every connection in progress", () => {
    expect(statusLabel("terminal", { state: "connecting" })).toBe(status.connecting);
    expect(statusLabel("rdp-external", { state: "connecting" })).toBe(status.connecting);
  });

  it("says where a connected session lives", () => {
    expect(statusLabel("terminal", { state: "connected" })).toBe(status.connected);
    expect(statusLabel("rdp-external", { state: "connected" })).toBe(status.rdpRunning);
    expect(statusLabel("external", { state: "connected" })).toBe(status.externalOpen);
  });

  it("names the exit code of a session that is over", () => {
    expect(statusLabel("terminal", { state: "ended", code: 0 })).toBe(
      interpolate(status.endedWithCode, { code: 0 }),
    );
    expect(statusLabel("terminal", { state: "failed", code: 255, reason: "rifiutata" })).toBe(
      interpolate(status.failedWithCode, { code: 255 }),
    );
    expect(statusLabel("terminal", { state: "failed", code: null, reason: "persa" })).toBe(
      status.failed,
    );
  });
});

describe("sessionDot", () => {
  it("shows the phase of a session, and none once ended", () => {
    expect(sessionDot({ state: "connected" })).toBe("connected");
    expect(sessionDot({ state: "failed", code: 255, reason: "rifiutata" })).toBe("failed");
    expect(sessionDot({ state: "ended", code: 0 })).toBe("idle");
  });
});

describe("formatDuration", () => {
  it("shows hours, minutes and seconds", () => {
    expect(formatDuration(12_000)).toBe("00:00:12");
    expect(formatDuration((3600 + 2 * 60 + 5) * 1000 + 999)).toBe("01:02:05");
  });

  it("never goes below zero", () => {
    expect(formatDuration(-500)).toBe("00:00:00");
  });
});
