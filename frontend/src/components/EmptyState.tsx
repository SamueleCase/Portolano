import { type ReactNode, useId } from "react";

import styles from "./EmptyState.module.css";

interface EmptyStateProps {
  title: string;
  text: string;
  /** The large buttons, such as "Nuova connessione" and "Importa da Royal TSX". */
  actions: ReactNode;
}

/** The centered glass card of an empty document (inventario §4.26, §5.1). */
export function EmptyState({ title, text, actions }: EmptyStateProps) {
  const titleId = useId();
  return (
    <section className={styles.card} aria-labelledby={titleId}>
      <h2 id={titleId} className={styles.title}>
        {title}
      </h2>
      <p className={styles.text}>{text}</p>
      <div className={styles.actions}>{actions}</div>
    </section>
  );
}
