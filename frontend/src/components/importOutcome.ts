/**
 * The outcomes of an imported item (inventario §5.13): imported, partial
 * (a password skipped counts here), unsupported, skipped (only the trash).
 */
export type ImportOutcome = "imported" | "partial" | "unsupported" | "skipped";
