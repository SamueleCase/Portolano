import { CountBadge } from "../../components/CountBadge";
import { ModeChip } from "../../components/ModeChip";
import { SecretBadge, type SecretState } from "../../components/SecretBadge";
import { SeverityMarker } from "../../components/SeverityMarker";
import { SourceChip } from "../../components/SourceChip";
import { type BadgeStatus, StatusBadge } from "../../components/StatusBadge";
import { type DotState, StatusDot } from "../../components/StatusDot";
import { Tag } from "../../components/Tag";
import { gallery } from "../../i18n/gallery";
import { texts } from "../../i18n/it";
import { sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;
const secretStates: SecretState[] = ["present", "absent", "staged", "locked"];
const statuses: BadgeStatus[] = ["connected", "connecting", "error", "warning", "info"];
const dots: DotState[] = ["connecting", "connected", "failed", "detail", "validation"];
const ignore = () => undefined;

/** Secret and status badges, counters, severity, dots, tags and chips. */
export function BadgesSection() {
  return (
    <GallerySection id="badges" title={gallery.section.badges}>
      <div className={styles.grid}>
        <Specimen label={caption.secretKeychain}>
          {secretStates.map((state) => (
            <SecretBadge key={state} state={state} backend="keychain" />
          ))}
        </Specimen>
        <Specimen label={caption.secretFile}>
          {secretStates.map((state) => (
            <SecretBadge key={state} state={state} backend="file" />
          ))}
        </Specimen>
        <Specimen label={caption.secretCompact}>
          <SecretBadge state="absent" backend="keychain" variant="compact" />
          <SecretBadge state="locked" backend="file" variant="compact" />
        </Specimen>
        <Specimen label={caption.secretText}>
          {secretStates.map((state) => (
            <SecretBadge key={state} state={state} backend="keychain" variant="text" />
          ))}
        </Specimen>
        <Specimen label={caption.statusBadges}>
          {statuses.map((status) => (
            <StatusBadge key={status} status={status} />
          ))}
        </Specimen>
        <Specimen label={caption.counts}>
          <CountBadge severity="error" count={1} />
          <CountBadge severity="warning" count={2} />
          <CountBadge severity="info" count={1} />
        </Specimen>
        <Specimen label={caption.severity}>
          <SeverityMarker severity="error" withLabel />
          <SeverityMarker severity="warning" withLabel />
          <SeverityMarker severity="info" withLabel />
        </Specimen>
        <Specimen label={caption.dots}>
          {dots.map((state) => (
            <StatusDot key={state} state={state} />
          ))}
          <StatusDot state="connected" size="large" />
        </Specimen>
        <Specimen label={caption.tagsChips}>
          <Tag name="postgres" />
          <Tag name="replica" variant="detail" />
          <Tag name="web" variant="editor" onRemove={ignore} />
          <ModeChip>{texts.components.credentialModes.inherit.title}</ModeChip>
          <ModeChip variant="detail">{sample.inherited}</ModeChip>
          <SourceChip source="tree" />
          <SourceChip source="address" />
        </Specimen>
      </div>
    </GallerySection>
  );
}
