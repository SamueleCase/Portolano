import { classNames } from "./classNames";
import styles from "./Chip.module.css";
import { visibleText } from "./visibleText";

interface ModeChipProps {
  /**
   * The mode or origin of a credential, such as "Ereditata dalla cartella
   * Produzione": it can carry names of the document.
   */
  children: string;
  /** `detail`: the larger chip of the node detail. */
  variant?: "field" | "detail";
}

/** The mode or origin of a credential (inventario §4.12). */
export function ModeChip({ children, variant = "field" }: ModeChipProps) {
  return (
    <span className={classNames(styles.mode, variant === "detail" && styles.modeDetail)}>
      {visibleText(children)}
    </span>
  );
}
