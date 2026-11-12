import type { ReactNode } from "react";

import { GlassPanel } from "../components/GlassPanel";
import styles from "./Gallery.module.css";

/** A section of the gallery: one screenshot each in the e2e. */
export function GallerySection({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <GlassPanel as="section" id={id} aria-labelledby={`${id}-title`} className={styles.section}>
      <h2 id={`${id}-title`} className={styles.sectionTitle}>
        {title}
      </h2>
      {children}
    </GlassPanel>
  );
}

/** One specimen with its caption. */
export function Specimen({ label, children }: { label: string; children: ReactNode }) {
  return (
    <figure className={styles.specimen}>
      <figcaption className={styles.caption}>{label}</figcaption>
      <div className={styles.sample}>{children}</div>
    </figure>
  );
}
