import type { TokenName } from "./tokens";

/** What a pair has to reach (SPEC-UI §1.6). */
export type ContrastKind = "text" | "control";

/** The WCAG minimum of each kind: 4.5:1 for text, 3:1 for control outlines. */
export const contrastMinimum: Record<ContrastKind, number> = { text: 4.5, control: 3 };

/** A foreground the interface draws on a surface. */
export interface ContrastPair {
  usage: string;
  kind: ContrastKind;
  /** The layers of the foreground, from the bottom up, drawn on `surface`. */
  foreground: readonly TokenName[];
  /** The surface, from the opaque bottom layer up. */
  surface: readonly TokenName[];
}

/** The surfaces of the window, from the opaque bottom up (inventario §7). */
export const surfaces = {
  panel: ["--color-bg", "--color-glass-panel"],
  dialog: ["--color-bg", "--color-glass-modal"],
  popover: ["--color-bg", "--color-popover"],
  fieldOnPanel: ["--color-bg", "--color-glass-panel", "--color-field"],
  fieldOnDialog: ["--color-bg", "--color-glass-modal", "--color-field"],
  toggleOffOnPanel: ["--color-bg", "--color-glass-panel", "--color-toggle-track-off"],
  toggleOffOnDialog: ["--color-bg", "--color-glass-modal", "--color-toggle-track-off"],
  hover: ["--color-hover"],
  active: ["--color-active"],
  terminal: ["--color-terminal"],
  accent: ["--color-accent"],
  accentHover: ["--color-accent-hover"],
  error: ["--color-error"],
  errorHover: ["--color-error-hover"],
} as const satisfies Record<string, readonly TokenName[]>;

type SurfaceName = keyof typeof surfaces;

function on(
  usage: string,
  kind: ContrastKind,
  foreground: TokenName | readonly TokenName[],
  surfaceNames: readonly SurfaceName[],
): ContrastPair[] {
  const layers = typeof foreground === "string" ? [foreground] : foreground;
  return surfaceNames.map((surface) => ({
    usage: `${usage} su ${surface}`,
    kind,
    foreground: layers,
    surface: surfaces[surface],
  }));
}

const textSurfaces: readonly SurfaceName[] = [
  "panel",
  "dialog",
  "popover",
  "fieldOnPanel",
  "fieldOnDialog",
  "hover",
  "active",
  "terminal",
];

/** Every foreground/surface pair the components use. */
export const contrastPairs: readonly ContrastPair[] = [
  ...on("testo principale", "text", "--color-text-primary", textSurfaces),
  ...on("testo secondario", "text", "--color-text-secondary", textSurfaces),
  ...on("testo attenuato, etichette e segnaposto", "text", "--color-text-muted", textSurfaces),
  ...on("testo d'accento e link", "text", "--color-accent-text", textSurfaces),
  ...on("testo su accento", "text", "--color-text-on-accent", ["accent", "accentHover"]),
  ...on("testo d'errore", "text", "--color-error-text", textSurfaces),
  ...on("testo sul bottone distruttivo", "text", "--color-text-on-error", ["error", "errorHover"]),
  ...on("avviso", "text", "--color-warn", ["panel", "dialog", "fieldOnPanel", "fieldOnDialog"]),
  ...on("successo", "text", "--color-ok", ["panel", "dialog", "fieldOnPanel", "fieldOnDialog"]),
  ...on("nota", "text", "--color-info", ["panel", "dialog", "fieldOnPanel", "fieldOnDialog"]),
  ...on("testo non supportato", "text", "--color-text-unsupported", ["panel", "hover", "active"]),
  ...on(
    "bordo dei campi",
    "control",
    ["--color-field", "--color-border-control"],
    ["panel", "dialog"],
  ),
  ...on("binario del toggle spento", "control", "--color-toggle-track-off", ["panel", "dialog"]),
  ...on("pomello del toggle spento", "control", "--color-toggle-knob-off", [
    "toggleOffOnPanel",
    "toggleOffOnDialog",
  ]),
  ...on("pomello del toggle acceso", "control", "--color-toggle-knob-on", ["accent"]),
  ...on("anello del fuoco", "control", "--color-accent-text", ["panel", "dialog", "active"]),
  ...on("icona non supportato", "control", "--color-node-unsupported", ["panel", "fieldOnPanel"]),
];
