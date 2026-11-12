import type { BackgroundControl } from "./backgroundControl";
import type { ConstellationFrame } from "./constellation";

/** What the animator needs from the window, so that tests can drive it. */
export interface AnimatorEnvironment {
  now(): number;
  requestFrame(callback: (time: number) => void): number;
  cancelFrame(handle: number): void;
  isHidden(): boolean;
  hasFocus(): boolean;
  systemReducedMotion(): boolean;
  /** Calls `onChange` when visibility, focus or the system motion setting change. */
  subscribe(onChange: () => void): () => void;
}

/** Where the animator paints: the canvas of `BackgroundCanvas`. */
export interface ConstellationSurface {
  /** Moves the points by `frame.step` and draws them. */
  draw(frame: ConstellationFrame): void;
  clear(): void;
}

/**
 * - `running`: the frame loop is active;
 * - `static`: reduce motion or "ridotto", one still drawing, no loop;
 * - `paused`: the window has no focus, the last drawing stays (DEC-028.6);
 * - `hidden`: nothing is drawn;
 * - `off`: only the background color.
 */
export type AnimatorState = "running" | "static" | "paused" | "hidden" | "off";

/**
 * A frame is skipped if less than this has passed since the last drawing
 * (DEC-028.6): 30 drawings per second on 60 and 120 Hz displays. Exactly
 * 1000/30 ms would often skip to every third frame at 60 Hz, as two frame
 * intervals can add up to a hair less.
 */
export const MIN_FRAME_INTERVAL_MS = 33;
const MAX_STEP_SECONDS = 0.1;
const DEFAULT_INTENSITY = 8;
const ALPHA_GAIN = 1.25;
const STATIC_ALPHA = 0.6 * ALPHA_GAIN;
const WORKING_CALM = 0.25;
const WORKING_WINDOW_MS = 10_000;
const CALM_RATE = 2.5;
const DIALOG_SPEED = 0.25;

/**
 * Runs the constellation of inventario §3.1 with the rules of DEC-028.6:
 * a loop of at most 30 fps only while the window is visible and focused,
 * one still drawing with reduced motion, nothing when hidden or off.
 */
export class BackgroundAnimator {
  private state: AnimatorState | undefined;
  private frameHandle: number | undefined;
  private lastFrame = 0;
  private time = 0;
  private calm = 1;
  private painted: "animated" | "static" | undefined;
  private readonly unsubscribe: Array<() => void>;

  constructor(
    private readonly surface: ConstellationSurface,
    private readonly control: BackgroundControl,
    private readonly environment: AnimatorEnvironment,
    private readonly intensity = DEFAULT_INTENSITY,
  ) {
    const update = () => {
      this.update();
    };
    this.unsubscribe = [control.subscribe(update), environment.subscribe(update)];
    this.update();
  }

  /** The current state, for tests and diagnostics. */
  current(): AnimatorState | undefined {
    return this.state;
  }

  /** The canvas lost its drawing (a resize): paint it again if nothing will. */
  repaint(): void {
    this.painted = undefined;
    this.update();
  }

  dispose(): void {
    this.stopLoop();
    this.unsubscribe.forEach((unsubscribe) => {
      unsubscribe();
    });
  }

  private wantedState(): AnimatorState {
    const { mode, reduceMotion } = this.control.inputs();
    if (mode === "off") return "off";
    if (this.environment.isHidden()) return "hidden";
    if (mode === "reduced" || reduceMotion || this.environment.systemReducedMotion()) {
      return "static";
    }
    return this.environment.hasFocus() ? "running" : "paused";
  }

  private update(): void {
    const state = this.wantedState();
    this.state = state;
    if (state === "running") {
      this.startLoop();
      return;
    }
    this.stopLoop();
    if (state === "off") {
      this.surface.clear();
      this.painted = undefined;
    } else if (state === "static" && this.painted !== "static") {
      this.paint("static", { alpha: STATIC_ALPHA, step: 0, time: this.time });
    } else if (state === "paused" && this.painted === undefined) {
      this.paint("animated", { alpha: this.animatedAlpha(), step: 0, time: this.time });
    }
  }

  private startLoop(): void {
    if (this.frameHandle !== undefined) return;
    this.lastFrame = this.environment.now();
    this.frameHandle = this.environment.requestFrame(this.tick);
  }

  private stopLoop(): void {
    if (this.frameHandle === undefined) return;
    this.environment.cancelFrame(this.frameHandle);
    this.frameHandle = undefined;
  }

  private readonly tick = (now: number): void => {
    this.frameHandle = this.environment.requestFrame(this.tick);
    const elapsed = now - this.lastFrame;
    if (elapsed < MIN_FRAME_INTERVAL_MS) return;
    this.lastFrame = now;
    const seconds = Math.min(MAX_STEP_SECONDS, Math.max(0, elapsed) / 1000);
    const inputs = this.control.inputs();
    const working =
      inputs.sessionTabActive &&
      this.control.now() - inputs.lastSessionActivity < WORKING_WINDOW_MS;
    this.calm += ((working ? WORKING_CALM : 1) - this.calm) * Math.min(1, seconds * CALM_RATE);
    const dialogSpeed = inputs.openDialogs > 0 ? DIALOG_SPEED : 1;
    const speed = (this.intensity / DEFAULT_INTENSITY) * this.calm * dialogSpeed;
    this.time += seconds * speed;
    this.paint("animated", { alpha: this.animatedAlpha(), step: seconds * speed, time: this.time });
  };

  private animatedAlpha(): number {
    return (0.35 + 0.65 * this.calm) * ALPHA_GAIN;
  }

  private paint(kind: "animated" | "static", frame: ConstellationFrame): void {
    this.surface.draw(frame);
    this.painted = kind;
  }
}
