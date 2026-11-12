import { describe, expect, it } from "vitest";

import {
  type AnimatorEnvironment,
  BackgroundAnimator,
  type ConstellationSurface,
} from "./animator";
import { createBackgroundControl } from "./backgroundControl";
import type { ConstellationFrame } from "./constellation";

/** A window with a manual clock and manual animation frames. */
class FakeWindow implements AnimatorEnvironment {
  clock = 0;
  hidden = false;
  focused = true;
  reducedMotion = false;
  private frames = new Map<number, (time: number) => void>();
  private nextHandle = 1;
  private listeners = new Set<() => void>();

  now = () => this.clock;
  requestFrame = (callback: (time: number) => void) => {
    const handle = this.nextHandle++;
    this.frames.set(handle, callback);
    return handle;
  };
  cancelFrame = (handle: number) => {
    this.frames.delete(handle);
  };
  isHidden = () => this.hidden;
  hasFocus = () => this.focused;
  systemReducedMotion = () => this.reducedMotion;
  subscribe = (onChange: () => void) => {
    this.listeners.add(onChange);
    return () => this.listeners.delete(onChange);
  };

  pendingFrames(): number {
    return this.frames.size;
  }

  /** Advances the clock by one display refresh and runs the frames due. */
  refresh(milliseconds: number): void {
    this.clock += milliseconds;
    const due = [...this.frames.values()];
    this.frames.clear();
    due.forEach((callback) => {
      callback(this.clock);
    });
  }

  /** Runs `seconds` of a 60 Hz display. */
  run(seconds: number): void {
    for (let elapsed = 0; elapsed < seconds * 1000; elapsed += 1000 / 60) {
      this.refresh(1000 / 60);
    }
  }

  change(update: Partial<Pick<FakeWindow, "hidden" | "focused" | "reducedMotion">>): void {
    Object.assign(this, update);
    this.listeners.forEach((listener) => {
      listener();
    });
  }
}

class RecordingSurface implements ConstellationSurface {
  frames: ConstellationFrame[] = [];
  clears = 0;
  draw(frame: ConstellationFrame) {
    this.frames.push(frame);
  }
  clear() {
    this.clears += 1;
  }
}

function setup(environment = new FakeWindow()) {
  const control = createBackgroundControl(environment.now);
  const surface = new RecordingSurface();
  const animator = new BackgroundAnimator(surface, control, environment);
  return { environment, control, surface, animator };
}

describe("BackgroundAnimator", () => {
  it("draws at most 30 times per second on a 60 Hz display", () => {
    const { environment, surface } = setup();

    environment.run(2);

    expect(surface.frames.length).toBeGreaterThanOrEqual(58);
    expect(surface.frames.length).toBeLessThanOrEqual(60);
  });

  it("draws at most 30 times per second on a 120 Hz display", () => {
    const { environment, surface } = setup();

    for (let frame = 0; frame < 240; frame++) environment.refresh(1000 / 120);

    expect(surface.frames.length).toBeLessThanOrEqual(60);
  });

  it("stops drawing and requesting frames while the window is hidden", () => {
    const { environment, surface, animator } = setup();
    environment.run(1);
    const drawn = surface.frames.length;

    environment.change({ hidden: true });
    environment.run(1);

    expect(animator.current()).toBe("hidden");
    expect(environment.pendingFrames()).toBe(0);
    expect(surface.frames).toHaveLength(drawn);
  });

  it("stops while the window has no focus (DEC-028.6)", () => {
    const { environment, surface, animator } = setup();
    environment.run(1);
    const drawn = surface.frames.length;

    environment.change({ focused: false });
    environment.run(1);

    expect(animator.current()).toBe("paused");
    expect(environment.pendingFrames()).toBe(0);
    expect(surface.frames).toHaveLength(drawn);
  });

  it("starts again when the window gets the focus back", () => {
    const { environment, surface } = setup();
    environment.change({ focused: false });

    environment.change({ focused: true });
    environment.run(1);

    expect(surface.frames.length).toBeGreaterThan(20);
  });

  it("draws one still frame without focus when nothing was drawn yet", () => {
    const environment = new FakeWindow();
    environment.focused = false;

    const { surface } = setup(environment);

    expect(surface.frames).toHaveLength(1);
    expect(surface.frames[0]?.step).toBe(0);
  });

  it("draws a single dimmed still frame with the system reduced motion", () => {
    const environment = new FakeWindow();
    environment.reducedMotion = true;

    const { surface, animator } = setup(environment);
    environment.run(2);

    expect(animator.current()).toBe("static");
    expect(environment.pendingFrames()).toBe(0);
    expect(surface.frames).toEqual([{ alpha: 0.75, step: 0, time: 0 }]);
  });

  it("draws a single still frame with the reduce motion preference", () => {
    const { environment, control, surface } = setup();

    control.setPreferences({ mode: "on", reduceMotion: true });
    environment.run(2);

    expect(surface.frames).toHaveLength(1);
    expect(surface.frames[0]).toMatchObject({ alpha: 0.75, step: 0 });
  });

  it("draws a single still frame with the background set to reduced", () => {
    const { environment, control, surface, animator } = setup();

    control.setPreferences({ mode: "reduced", reduceMotion: false });
    environment.run(1);

    expect(animator.current()).toBe("static");
    expect(surface.frames).toHaveLength(1);
  });

  it("clears the canvas and draws nothing with the background off", () => {
    const { environment, control, surface, animator } = setup();

    control.setPreferences({ mode: "off", reduceMotion: false });
    environment.run(1);

    expect(animator.current()).toBe("off");
    expect(surface.clears).toBe(1);
    expect(surface.frames).toHaveLength(0);
    expect(environment.pendingFrames()).toBe(0);
  });

  it("moves at a quarter of the speed while a dialog is open (ambiguity 4)", () => {
    const { environment, control, surface } = setup();
    environment.run(1);
    const before = surface.frames.at(-1)?.step ?? 0;

    control.openDialog();
    environment.run(1);

    expect(surface.frames.at(-1)?.step).toBeCloseTo(before * 0.25, 5);
  });

  it("goes back to full speed when the dialog closes", () => {
    const { environment, control, surface } = setup();
    environment.run(1);
    const before = surface.frames.at(-1)?.step ?? 0;
    const close = control.openDialog();
    environment.run(1);

    close();
    environment.run(1);

    expect(surface.frames.at(-1)?.step).toBeCloseTo(before, 5);
  });

  it("calms down and dims while the user works in a session", () => {
    const { environment, control, surface } = setup();
    environment.run(1);
    const rest = surface.frames.at(-1);

    control.setSessionTabActive(true);
    control.noteSessionActivity();
    environment.run(3);

    const working = surface.frames.at(-1);
    expect(working?.alpha).toBeLessThan(0.7);
    expect(working?.step).toBeLessThan((rest?.step ?? 0) * 0.3);
  });

  it("is full again ten seconds after the last activity in the session", () => {
    const { environment, control, surface } = setup();
    control.setSessionTabActive(true);
    control.noteSessionActivity();

    environment.run(13);

    expect(surface.frames.at(-1)?.alpha).toBeCloseTo(1.25, 2);
  });

  it("paints the still frame again after the canvas is resized", () => {
    const { control, surface, animator } = setup();
    control.setPreferences({ mode: "reduced", reduceMotion: false });

    animator.repaint();

    expect(surface.frames).toHaveLength(2);
  });

  it("cancels its frame when disposed", () => {
    const { environment, animator } = setup();

    animator.dispose();

    expect(environment.pendingFrames()).toBe(0);
  });
});
