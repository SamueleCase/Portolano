import { texts } from "../../i18n/it";
import styles from "./Sidebar.module.css";

/** The rows shown while the tree loads (SPEC-UI §5.4 "Caricamenti"). */
const ROWS = [0, 1, 1, 1, 0, 1] as const;

/** A skeleton of the tree, announced as busy. */
export function TreeSkeleton() {
  return (
    <div role="status" aria-busy="true" aria-label={texts.tree.loading} className={styles.skeleton}>
      {ROWS.map((depth, row) => (
        <span key={row} className={depth === 0 ? styles.bone : styles.boneNested} />
      ))}
    </div>
  );
}
