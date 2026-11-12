/**
 * Design tokens as CSS custom properties (SPEC-UI §5.2), with the values of
 * `docs/ui/inventario.md` §2. UI-WP0 defines only those its test screen
 * uses; UI-WP1 completes the set.
 */
export const tokens = {
  "--color-bg": "#1B1A17",
  "--color-text-primary": "#EDE9E1",
  "--color-text-secondary": "#CFC9BD",
  "--color-text-muted": "#A8A297",
  "--color-accent-text": "#7FC2A0",
  // `--err` lightened for text, which needs 4.5:1 (inventario §7).
  "--color-error-text": "#EE8279",
  "--font-ui": "'Hanken Grotesk', system-ui, sans-serif",
  "--font-mono": "'IBM Plex Mono', ui-monospace, monospace",
  "--font-size-base": "13.5px",
  "--font-size-mono-large": "14px",
  "--font-size-panel-title": "20px",
  "--font-weight-regular": "400",
  "--font-weight-strong": "600",
  "--space-2": "8px",
  "--space-4": "16px",
  "--space-7": "32px",
} as const;

/** Defines the tokens on `root`, for the stylesheets to use. */
export function applyTokens(root: HTMLElement): void {
  for (const [name, value] of Object.entries(tokens)) {
    root.style.setProperty(name, value);
  }
}
