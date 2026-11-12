import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./Tag.module.css";
import { visibleText } from "./visibleText";

interface TagProps {
  /** A tag of the document: untrusted text. */
  name: string;
  /** `detail`: on the node header; `editor`: 12.5 px with "×". */
  variant?: "tree" | "detail" | "editor";
  onRemove?: (() => void) | undefined;
}

/** A tag of a node (inventario §4.12). */
export function Tag({ name, variant = "tree", onRemove }: TagProps) {
  const shown = visibleText(name);
  return (
    <span className={classNames(styles.tag, styles[variant])}>
      {shown}
      {onRemove && (
        <button
          type="button"
          className={styles.remove}
          aria-label={interpolate(texts.components.tagInput.remove, { tag: shown })}
          onClick={onRemove}
        >
          ×
        </button>
      )}
    </span>
  );
}
