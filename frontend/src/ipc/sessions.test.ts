import { describe, expect, it } from "vitest";

import { decodeFrame, FRAME_VERSION, FrameKind } from "./sessions";

function buffer(...bytes: number[]): ArrayBuffer {
  return new Uint8Array(bytes).buffer;
}

describe("decodeFrame", () => {
  it("splits the header from the payload", () => {
    const frame = decodeFrame(buffer(FRAME_VERSION, FrameKind.terminalOutput, 104, 105));

    expect(frame?.kind).toBe(FrameKind.terminalOutput);
    expect(new TextDecoder().decode(frame?.payload)).toBe("hi");
  });

  it("ignores a frame of another version", () => {
    expect(decodeFrame(buffer(FRAME_VERSION + 1, FrameKind.terminalOutput, 104))).toBeNull();
  });

  it("ignores a frame without header", () => {
    expect(decodeFrame(buffer(FRAME_VERSION))).toBeNull();
  });
});
