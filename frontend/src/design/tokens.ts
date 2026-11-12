/**
 * Design tokens as CSS custom properties (SPEC-UI §5.2), with the values of
 * `docs/ui/inventario.md` §2 (palette 2a) under semantic names (ambiguity 1).
 * The values below AA in inventario §7 are corrected keeping their hue; the
 * pairs the interface uses are checked by `contrast.test.ts`.
 *
 * This is the only place with literal colors and measures: stylesheets use
 * the custom properties.
 */
export const tokens = {
  // Surfaces (inventario §2.1, §2.11).
  "--color-bg": "#1B1A17",
  "--color-glass-panel": "rgba(58,54,48,.16)",
  "--color-glass-modal": "rgba(40,37,33,.72)",
  "--color-popover": "rgba(40,37,33,.92)",
  "--color-terminal": "#0C0B0A",
  "--color-terminal-border": "rgba(255,255,255,.08)",
  "--color-field": "rgba(255,255,255,.07)",
  "--color-border": "rgba(255,245,225,.16)",
  // The border of fields, selects and segmented controls: `--color-border`
  // is 1.6:1 on a panel, under the 3:1 of WCAG 1.4.11 (inventario §7).
  "--color-border-control": "rgba(255,245,225,.42)",
  "--color-hover": "#2A2723",
  "--color-active": "#24382E",
  "--color-overlay-panel": "rgba(0,0,0,.4)",
  "--color-overlay-dialog": "rgba(0,0,0,.45)",
  "--color-surface-inset": "rgba(255,255,255,.04)",
  "--color-surface-subtle": "rgba(255,255,255,.05)",
  "--color-surface-raised": "rgba(255,255,255,.08)",
  "--color-surface-chip": "rgba(255,255,255,.1)",
  "--color-track": "rgba(255,255,255,.12)",
  // The track of a switched-off toggle: `.16` is 1.7:1 on a dialog.
  "--color-toggle-track-off": "rgba(255,255,255,.34)",
  "--color-toggle-knob-on": "#F5F1E8",
  "--color-toggle-knob-off": "#CFC9BD",
  "--color-skipped": "rgba(255,255,255,.25)",
  "--color-skipped-marker": "rgba(255,255,255,.35)",

  // Text (inventario §2.2: the DS scheme, secondary is the lighter one).
  "--color-text-primary": "#EDE9E1",
  "--color-text-secondary": "#CFC9BD",
  "--color-text-muted": "#A8A297",
  "--color-text-on-accent": "#F5F1E8",
  "--color-text-on-error": "#1B0F0D",

  // Accent (inventario §2.3). The fill is darker than `#2F7D5B` (4.43:1 with
  // the text on it) and its hover darker still, not lighter (3.43:1).
  "--color-accent": "#2B7454",
  "--color-accent-hover": "#25664A",
  "--color-accent-text": "#7FC2A0",

  // Semantic colors (inventario §2.4). `--color-error` stays for fills,
  // borders and markers; text uses `--color-error-text` (4.2:1 on a field).
  "--color-ok": "#9BD06E",
  "--color-warn": "#E0A84B",
  "--color-error": "#E5695F",
  "--color-error-hover": "#EE8279",
  "--color-error-text": "#EE8279",
  "--color-info": "#7FB2D9",

  // Node types and protocols (inventario §2.5). The unsupported color is
  // for icons; its text needs a lighter shade (4.46:1 on a panel).
  "--color-node-ssh": "#7FC2A0",
  "--color-node-rdp": "#7FA6D9",
  "--color-node-credential": "#D9B45F",
  "--color-node-folder": "#A8A297",
  "--color-node-unsupported": "#8A8479",
  "--color-text-unsupported": "#A39D92",

  // The colors a node can take (ambiguity 15).
  "--color-swatch-1": "#7FC2A0",
  "--color-swatch-2": "#7FA6D9",
  "--color-swatch-3": "#D9B45F",
  "--color-swatch-4": "#D98F7F",
  "--color-swatch-5": "#B79FD9",
  "--color-swatch-6": "#A8A297",

  // Typography (inventario §2.7).
  "--font-ui": "'Hanken Grotesk', system-ui, sans-serif",
  "--font-mono": "'IBM Plex Mono', ui-monospace, monospace",
  "--font-size-hero": "64px",
  "--font-size-tile": "34px",
  "--font-size-node-title": "26px",
  "--font-size-empty-title": "24px",
  "--font-size-logo-large": "22px",
  "--font-size-counter": "22px",
  "--font-size-panel-title": "20px",
  "--font-size-dialog-title": "18px",
  "--font-size-logo": "16px",
  "--font-size-section-title": "15px",
  "--font-size-lead": "14.5px",
  "--font-size-mono-large": "14px",
  "--font-size-base": "13.5px",
  "--font-size-button": "13px",
  "--font-size-button-large": "13.5px",
  "--font-size-body": "13px",
  "--font-size-secondary": "12.5px",
  "--font-size-label": "12px",
  "--font-size-group": "11.5px",
  "--font-size-small": "11px",
  "--font-size-chevron": "10px",
  "--font-weight-regular": "400",
  "--font-weight-medium": "500",
  "--font-weight-strong": "600",
  "--line-height-tight": "1",
  "--line-height-default": "1.4",
  "--line-height-message": "1.45",
  "--line-height-body": "1.5",
  "--line-height-lead": "1.55",
  "--line-height-code": "1.6",
  "--line-height-terminal": "1.65",
  "--tracking-hero": "-.04em",
  "--tracking-tile": "-.03em",
  "--tracking-title": "-.02em",
  "--tracking-panel": "-.01em",
  "--tracking-group": ".1em",
  "--tracking-password": ".2em",
  "--tracking-handle": "-2px",

  // Spacing: the DS scale (inventario §2.8) ...
  "--space-1": "4px",
  "--space-2": "8px",
  "--space-3": "12px",
  "--space-4": "16px",
  "--space-5": "20px",
  "--space-6": "24px",
  "--space-7": "32px",
  "--space-8": "48px",
  // ... and the off-scale values the mock uses, reproduced exactly.
  "--space-fine-1": "1px",
  "--space-fine-2": "2px",
  "--space-fine-3": "3px",
  "--space-fine-5": "5px",
  "--space-fine-6": "6px",
  "--space-fine-7": "7px",
  "--space-fine-9": "9px",
  "--space-fine-10": "10px",
  "--space-fine-11": "11px",
  "--space-fine-13": "13px",
  "--space-fine-14": "14px",
  "--space-fine-18": "18px",
  "--space-fine-22": "22px",
  "--space-fine-26": "26px",
  "--space-fine-28": "28px",
  "--space-fine-30": "30px",
  "--space-fine-36": "36px",

  // Sizes of the components (inventario §4).
  "--size-control": "38px",
  "--size-text-area": "52px",
  "--size-row": "34px",
  "--size-picker-row": "32px",
  "--size-indent": "18px",
  "--size-dot": "7px",
  "--size-dot-large": "9px",
  "--size-marker": "8px",
  "--size-route-point": "11px",
  "--size-logo": "24px",
  "--size-logo-large": "30px",
  "--size-icon-badge": "13px",
  "--size-icon-vault": "14px",
  "--size-icon-list": "15px",
  "--size-icon": "16px",
  "--size-icon-tile": "20px",
  "--size-icon-editor": "22px",
  "--size-icon-detail": "26px",
  "--stroke-logo": "4",
  "--stroke-icon": "1.8",
  "--stroke-icon-tile": "1.6",
  "--stroke-icon-badge": "2",
  "--stroke-icon-large": "1.5",
  "--size-spinner": "12px",
  "--size-swatch": "20px",
  "--size-toggle-width": "34px",
  "--size-toggle-height": "20px",
  "--size-toggle-knob": "16px",
  "--size-tile-large": "52px",
  "--size-tile": "44px",
  "--size-numeric-field": "56px",
  "--size-placeholder": "38px",
  "--size-connector": "24px",
  "--size-connector-short": "18px",
  "--size-connector-inherit": "20px",
  "--size-line": "2px",
  "--size-route-line": "3px",
  "--size-progress": "4px",
  "--size-minibar": "6px",
  "--size-stacked-bar": "10px",
  "--size-mode-list": "270px",
  "--size-picker-column": "220px",
  "--size-sidebar-max": "320px",
  "--size-label-column": "150px",
  "--size-session-label-column": "140px",
  "--size-import-label-column": "90px",
  "--size-tag-input": "80px",
  "--size-remove-column": "20px",
  "--size-secret-label": "110px",
  "--size-severity-column": "76px",
  "--size-report-column": "104px",
  "--size-key-type-column": "86px",
  "--size-preview-min-height": "300px",
  "--size-terminal-min-height": "240px",
  "--width-dialog-small": "440px",
  "--width-dialog-medium": "460px",
  "--width-dialog-wizard": "560px",
  "--width-dialog-preferences": "760px",
  "--width-dialog-selector": "820px",
  "--width-dialog-report": "860px",
  "--width-dialog-margin": "70px",
  "--width-side-panel": "620px",
  "--width-empty-state": "440px",
  "--width-toast": "360px",
  "--width-context-menu": "220px",
  "--width-tooltip": "260px",
  "--width-name-field": "260px",
  "--border-width": "1px",
  "--border-width-strong": "1.5px",
  "--border-width-focus": "2px",

  // Radii (inventario §2.9; 14 px is `--radius-panel`, ambiguity 2).
  "--radius-line": "2px",
  "--radius-route": "3px",
  "--radius-tiny": "4px",
  "--radius-bar": "6px",
  "--radius-segment": "7px",
  "--radius-small": "8px",
  "--radius-row": "9px",
  "--radius-control": "10px",
  "--radius-card": "12px",
  "--radius-panel": "14px",
  "--radius-surface": "18px",
  "--radius-pill": "999px",
  "--radius-round": "50%",

  // Shadows, glass and opacity (inventario §2.10–2.12).
  "--shadow-edge": "inset 0 1px 0 rgba(255,255,255,.08)",
  "--shadow-modal": "0 24px 60px rgba(0,0,0,.4)",
  "--shadow-drag": "0 8px 24px rgba(0,0,0,.35)",
  "--shadow-drop-target": "inset 0 2px 0 var(--color-accent-text)",
  "--blur-panel": "blur(6px) saturate(140%)",
  "--blur-modal": "blur(18px)",
  "--opacity-disabled": ".45",
  "--opacity-dragging": ".45",

  // Motion (inventario §2.14, §3.2).
  "--duration-fast": "160ms",
  "--duration-base": "260ms",
  "--duration-slow": "480ms",
  "--duration-spinner": "800ms",
  "--delay-tooltip": "500ms",
  "--ease-spring": "cubic-bezier(.34,1.56,.64,1)",
  "--ease-out": "cubic-bezier(.2,.8,.2,1)",
  "--motion-enter-scale": ".96",
  "--motion-press-scale": ".96",
  "--motion-enter-offset": "16px",
  "--motion-panel-offset": "100%",
  "--motion-shake-offset": "4px",
  "--motion-lift-rotation": "-1.2deg",

  // Stacking (inventario §2.13, the proposal).
  "--z-canvas": "0",
  "--z-app": "1",
  "--z-panel": "10",
  "--z-dialog": "20",
  "--z-popover": "30",
  "--z-toast": "40",
} as const;

/** The name of a design token. */
export type TokenName = keyof typeof tokens;

/** The compact density (ambiguity 8: rows 26 px and text 12.5 px). */
export const compactTokens = {
  "--size-row": "26px",
  "--font-size-base": "12.5px",
} as const satisfies Partial<Record<TokenName, string>>;

/**
 * Reduce motion (ambiguity 5): movements become fades of at most 120 ms, so
 * whatever enters or moves only changes its opacity.
 */
export const reducedMotionTokens = {
  "--duration-fast": "120ms",
  "--duration-base": "120ms",
  "--duration-slow": "120ms",
  "--ease-spring": "cubic-bezier(.2,.8,.2,1)",
  "--motion-enter-scale": "1",
  "--motion-press-scale": "1",
  "--motion-enter-offset": "0px",
  "--motion-panel-offset": "0px",
  "--motion-shake-offset": "0px",
  "--motion-lift-rotation": "0deg",
} as const satisfies Partial<Record<TokenName, string>>;

/** The colors of the background constellation, as RGB channels (§2.6). */
export const constellationColors = {
  line: "201,195,182",
  point: "127,194,160",
} as const;

function declarations(values: Readonly<Record<string, string>>): string {
  return Object.entries(values)
    .map(([name, value]) => `${name}:${value};`)
    .join("");
}

/**
 * The stylesheet with every token: the defaults, the compact density and
 * reduce motion, both from the preference (`data-*` on the root, see
 * `appearance.ts`) and from the system setting.
 */
export function tokenStylesheet(): string {
  const reduced = declarations(reducedMotionTokens);
  return [
    `:root{${declarations(tokens)}}`,
    `:root[data-density="compact"]{${declarations(compactTokens)}}`,
    `:root[data-motion="reduced"]{${reduced}}`,
    `@media (prefers-reduced-motion: reduce){:root{${reduced}}}`,
  ].join("\n");
}

const STYLESHEET_ID = "portolano-tokens";

/** Defines the tokens in the document of `root`, for the stylesheets to use. */
export function applyTokens(root: HTMLElement): void {
  const document = root.ownerDocument;
  if (document.getElementById(STYLESHEET_ID)) {
    return;
  }
  const style = document.createElement("style");
  style.id = STYLESHEET_ID;
  style.textContent = tokenStylesheet();
  document.head.append(style);
}
