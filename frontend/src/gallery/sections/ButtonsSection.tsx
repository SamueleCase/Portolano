import { Button, type ButtonVariant } from "../../components/Button";
import { gallery } from "../../i18n/gallery";
import { buttonLabels } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const variants: ButtonVariant[] = [
  "primary",
  "secondary",
  "toolbar",
  "ghost",
  "add",
  "danger",
  "danger-text",
  "small",
];

/** Every variant of `Button` at rest, disabled (ambiguity 7) and busy. */
export function ButtonsSection() {
  const { states } = gallery;
  return (
    <GallerySection id="buttons" title={gallery.section.buttons}>
      <div className={styles.grid}>
        {variants.map((variant) => (
          <Specimen key={variant} label={variant}>
            <Button variant={variant}>{buttonLabels[variant]}</Button>
            <Button variant={variant} disabled>
              {states.disabled}
            </Button>
            <Button variant={variant} loading>
              {states.loading}
            </Button>
          </Specimen>
        ))}
        <Specimen label={gallery.caption.buttonSizes}>
          <Button variant="primary" size="wide">
            {buttonLabels.primary}
          </Button>
          <Button variant="primary" size="large">
            {buttonLabels.large}
          </Button>
          <Button size="large">{buttonLabels.largeSecondary}</Button>
        </Specimen>
      </div>
    </GallerySection>
  );
}
