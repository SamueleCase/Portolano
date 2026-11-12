import type { AnimatorEnvironment } from "./animator";

/** The animator's view of a real window. */
export function windowEnvironment(target: Window): AnimatorEnvironment {
  const motion =
    typeof target.matchMedia === "function"
      ? target.matchMedia("(prefers-reduced-motion: reduce)")
      : undefined;
  return {
    now: () => target.performance.now(),
    requestFrame: (callback) => target.requestAnimationFrame(callback),
    cancelFrame: (handle) => {
      target.cancelAnimationFrame(handle);
    },
    isHidden: () => target.document.hidden,
    hasFocus: () => target.document.hasFocus(),
    systemReducedMotion: () => motion?.matches ?? false,
    subscribe(onChange) {
      target.document.addEventListener("visibilitychange", onChange);
      target.addEventListener("focus", onChange);
      target.addEventListener("blur", onChange);
      motion?.addEventListener("change", onChange);
      return () => {
        target.document.removeEventListener("visibilitychange", onChange);
        target.removeEventListener("focus", onChange);
        target.removeEventListener("blur", onChange);
        motion?.removeEventListener("change", onChange);
      };
    },
  };
}
