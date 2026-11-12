import { CommandPreview } from "../../components/CommandPreview";
import { ModeChip } from "../../components/ModeChip";
import { Tag } from "../../components/Tag";
import { visibleText } from "../../components/visibleText";
import { gallery } from "../../i18n/gallery";
import { hostileName, sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

/** A name with control and bidi characters, shown as the CLI shows it (DEC-026.4). */
export function UntrustedSection() {
  return (
    <GallerySection id="untrusted" title={gallery.section.untrusted}>
      <div className={styles.wide}>
        <Specimen label={gallery.caption.untrustedText}>
          <span data-testid="hostile-name">{visibleText(hostileName)}</span>
        </Specimen>
        <Specimen label={gallery.caption.untrustedComponents}>
          <Tag name={hostileName} />
          <ModeChip>{hostileName}</ModeChip>
        </Specimen>
        <Specimen label={gallery.caption.command}>
          <div className={styles.column}>
            <CommandPreview caption={sample.commandCaption} command={`ssh -- ${hostileName}`} />
          </div>
        </Specimen>
      </div>
    </GallerySection>
  );
}
