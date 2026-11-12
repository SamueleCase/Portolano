import type { CSSProperties } from "react";

import { Logo } from "../../components/Logo";
import type { TokenName } from "../../design/tokens";
import { gallery } from "../../i18n/gallery";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const colors: TokenName[] = [
  "--color-bg",
  "--color-text-primary",
  "--color-text-secondary",
  "--color-text-muted",
  "--color-accent",
  "--color-accent-hover",
  "--color-accent-text",
  "--color-text-on-accent",
  "--color-ok",
  "--color-warn",
  "--color-error",
  "--color-error-text",
  "--color-info",
  "--color-hover",
  "--color-active",
  "--color-terminal",
  "--color-node-ssh",
  "--color-node-rdp",
  "--color-node-credential",
  "--color-node-folder",
  "--color-node-unsupported",
  "--color-text-unsupported",
  "--color-border",
  "--color-border-control",
];

const typeScale: TokenName[] = [
  "--font-size-node-title",
  "--font-size-panel-title",
  "--font-size-dialog-title",
  "--font-size-base",
  "--font-size-secondary",
  "--font-size-label",
];

function swatch(name: TokenName): CSSProperties & { "--swatch": string } {
  return { "--swatch": `var(${name})` };
}

const caption = gallery.caption;

/** The logo, the palette and the type scale. */
export function FoundationsSection() {
  return (
    <GallerySection id="foundations" title={gallery.section.foundations}>
      <div className={styles.grid}>
        <Specimen label={caption.logoToolbar}>
          <Logo />
        </Specimen>
        <Specimen label={caption.logoFirstRun}>
          <Logo size="first-run" />
        </Specimen>
        <Specimen label={caption.mark}>
          <Logo wordmark={false} />
        </Specimen>
      </div>
      <Specimen label={caption.colors}>
        {colors.map((name) => (
          <span key={name} className={styles.swatch} style={swatch(name)}>
            <span className={styles.chip} />
            {name.replace("--color-", "")}
          </span>
        ))}
      </Specimen>
      <Specimen label={caption.typeScale}>
        <div className={styles.column}>
          {typeScale.map((size) => (
            <div key={size} className={styles.typeRow}>
              <span className={styles.typeName}>{size.replace("--font-size-", "")}</span>
              <span style={{ fontSize: `var(${size})` }}>{caption.typeSample}</span>
            </div>
          ))}
        </div>
      </Specimen>
    </GallerySection>
  );
}
