import type { ReactNode } from "react";

import styles from "./GroupHeader.module.css";

/**
 * A group heading: mono 11.5 px, uppercase, muted ("ROTTA", "DA DOVE
 * ARRIVERÀ", inventario §4.26). A real heading at `level`.
 */
export function GroupHeader({ level = 3, children }: { level?: 2 | 3 | 4; children: ReactNode }) {
  const Heading = `h${level}` as const;
  return <Heading className={styles.header}>{children}</Heading>;
}
