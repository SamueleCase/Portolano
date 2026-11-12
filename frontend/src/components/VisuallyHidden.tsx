import type { ReactNode } from "react";

import styles from "./VisuallyHidden.module.css";

/** Text for assistive technologies only, such as the meaning of a colored dot. */
export function VisuallyHidden({ children }: { children: ReactNode }) {
  return <span className={styles.hidden}>{children}</span>;
}
