import { useEffect, useRef } from "react";

import { BackgroundAnimator } from "./animator";
import styles from "./BackgroundCanvas.module.css";
import { type BackgroundControl, backgroundControl } from "./backgroundControl";
import { createCanvasSurface } from "./canvasSurface";
import { createPoints, seededRandom } from "./constellation";
import { windowEnvironment } from "./windowEnvironment";

interface BackgroundCanvasProps {
  control?: BackgroundControl;
  /** Fixes the sky, for screenshots; random otherwise. */
  seed?: number;
}

/**
 * The constellation behind the window (inventario §3.1, DEC-028.6). The
 * window is opaque: this canvas is what the glass panels blur. The terminal
 * is opaque and has no blur, so nothing of it shows there.
 */
export function BackgroundCanvas({ control = backgroundControl, seed }: BackgroundCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) {
      return undefined;
    }
    const random = seed === undefined ? Math.random : seededRandom(seed);
    const surface = createCanvasSurface(canvas, context, createPoints(random));
    const animator = new BackgroundAnimator(surface, control, windowEnvironment(window));
    const resize = new ResizeObserver(() => {
      animator.repaint();
    });
    resize.observe(canvas);
    return () => {
      resize.disconnect();
      animator.dispose();
    };
  }, [control, seed]);

  return <canvas ref={canvasRef} className={styles.canvas} aria-hidden="true" />;
}
