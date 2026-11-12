import { texts } from "../i18n/it";
import styles from "./Chip.module.css";

/** Where a jump host comes from: a node of the tree or a free address. */
export type JumpSource = "tree" | "address";

/** "dall'albero" or "indirizzo libero" in the jump host list (inventario §4.12). */
export function SourceChip({ source }: { source: JumpSource }) {
  return <span className={styles.source}>{texts.components.sortable.sources[source]}</span>;
}
