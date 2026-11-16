import type { FeatureRegistry } from "./features";

/**
 * The functions mounted at the hook points of the window (`features.ts`).
 * UI-WP5 (editor, password, vault, login, preferences) and UI-WP6 (import,
 * validation) add their line here when they integrate (DEC-035).
 */
export const featureRegistry: FeatureRegistry = {};
