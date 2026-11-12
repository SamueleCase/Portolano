import { useEffect } from "react";

/** The "Sfondo animato" preference (ambiguity 19). */
export type BackgroundMode = "on" | "reduced" | "off";

/** What the screens tell the background. */
export interface BackgroundInputs {
  mode: BackgroundMode;
  /** The "Riduci movimento" preference; the system setting is read apart. */
  reduceMotion: boolean;
  /** Dialogs open now: the background slows down behind them (ambiguity 4). */
  openDialogs: number;
  /** Whether the active tab is a session rather than the detail. */
  sessionTabActive: boolean;
  /** When the user last worked in a session (connection, key), on `now()`. */
  lastSessionActivity: number;
}

/**
 * The small API the screens use to drive the background (SPEC-UI §5.2): the
 * preferences, the open dialogs and the work in a session. It knows nothing
 * of the stores of the screens.
 */
export interface BackgroundControl {
  inputs(): BackgroundInputs;
  subscribe(listener: () => void): () => void;
  setPreferences(preferences: { mode: BackgroundMode; reduceMotion: boolean }): void;
  /** Counts a dialog as open until the returned function is called. */
  openDialog(): () => void;
  setSessionTabActive(active: boolean): void;
  /** A connection that succeeded or a key in the terminal. */
  noteSessionActivity(): void;
  now(): number;
}

/** A control with its own state; `now` is the clock of the animation. */
export function createBackgroundControl(now: () => number = () => performance.now()) {
  let inputs: BackgroundInputs = {
    mode: "on",
    reduceMotion: false,
    openDialogs: 0,
    sessionTabActive: false,
    lastSessionActivity: Number.NEGATIVE_INFINITY,
  };
  const listeners = new Set<() => void>();
  const update = (change: Partial<BackgroundInputs>) => {
    inputs = { ...inputs, ...change };
    listeners.forEach((listener) => {
      listener();
    });
  };

  const control: BackgroundControl = {
    inputs: () => inputs,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    setPreferences: (preferences) => {
      update(preferences);
    },
    openDialog() {
      update({ openDialogs: inputs.openDialogs + 1 });
      let open = true;
      return () => {
        if (open) {
          open = false;
          update({ openDialogs: inputs.openDialogs - 1 });
        }
      };
    },
    setSessionTabActive: (active) => {
      update({ sessionTabActive: active });
    },
    noteSessionActivity: () => {
      update({ lastSessionActivity: now() });
    },
    now,
  };
  return control;
}

/** The control of the window's background. */
export const backgroundControl = createBackgroundControl();

/** Slows the background down while a dialog is `open` (ambiguity 4). */
export function useBackgroundDialog(
  open: boolean,
  control: BackgroundControl = backgroundControl,
): void {
  useEffect(() => (open ? control.openDialog() : undefined), [open, control]);
}
