import { Button } from "../../components/Button";
import { Card } from "../../components/Card";
import { CommandPreview } from "../../components/CommandPreview";
import { DropZone } from "../../components/DropZone";
import { EmptyState } from "../../components/EmptyState";
import { GlassPanel } from "../../components/GlassPanel";
import { GroupHeader } from "../../components/GroupHeader";
import { IconTile } from "../../components/IconTile";
import { MiniBar } from "../../components/MiniBar";
import { ProgressBar } from "../../components/ProgressBar";
import { StackedBar } from "../../components/StackedBar";
import { StatTile } from "../../components/StatTile";
import { VaultStatus } from "../../components/VaultStatus";
import { gallery } from "../../i18n/gallery";
import { report, sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;
const ignore = () => undefined;

/** Glass, cards, empty state, command, tiles, drop zone, bars and the vault footer. */
export function SurfacesSection() {
  const total = report.segments.reduce((sum, segment) => sum + segment.count, 0);
  const largest = Math.max(...report.types.map((type) => type.value));
  return (
    <GallerySection id="surfaces" title={gallery.section.surfaces}>
      <div className={styles.grid}>
        <Specimen label={caption.glassInner}>
          <GlassPanel variant="inner" className={styles.innerSample}>
            <GroupHeader>{caption.groupHeader}</GroupHeader>
          </GlassPanel>
        </Specimen>
        <Specimen label={caption.cards}>
          <div className={styles.column}>
            <Card tinted>{sample.inherited}</Card>
            <Card tone="warning">{caption.cardWarning}</Card>
            <Card tone="error" large>
              {caption.cardError}
            </Card>
          </div>
        </Specimen>
        <Specimen label={caption.iconTiles}>
          <IconTile kind="ssh" color="#7FC2A0" />
          <IconTile kind="rdp" />
          <IconTile kind="credential" size="editor" />
          <IconTile kind="folder" size="editor" icon={{ source: "builtin", name: "cloud" }} />
        </Specimen>
        <Specimen label={caption.progress}>
          <div className={styles.column}>
            <ProgressBar label={sample.progress} value={3} max={12} valueText="3 / 12" />
          </div>
        </Specimen>
      </div>
      <div className={styles.wide}>
        <Specimen label={caption.emptyState}>
          <EmptyState
            title={sample.documentTitle}
            text={sample.documentText}
            actions={
              <>
                <Button variant="primary" size="large">
                  {sample.newConnection}
                </Button>
                <Button size="large">{sample.importRoyal}</Button>
              </>
            }
          />
        </Specimen>
        <Specimen label={caption.command}>
          <div className={styles.column}>
            <CommandPreview caption={sample.commandCaption} command={sample.command} />
          </div>
        </Specimen>
        <Specimen label={caption.dropZone}>
          <div className={styles.column}>
            <DropZone
              title={sample.dropTitle}
              hint={sample.dropHint}
              action={<Button>{sample.chooseFile}</Button>}
              onDrop={ignore}
            />
          </div>
        </Specimen>
        <Specimen label={caption.report}>
          <div className={styles.column}>
            <StackedBar label={sample.report} segments={report.segments} />
            <div className={styles.grid}>
              {report.segments.map((segment) => (
                <StatTile key={segment.outcome} {...segment} total={total} />
              ))}
            </div>
            {report.types.map((type) => (
              <MiniBar key={type.label} {...type} max={largest} />
            ))}
          </div>
        </Specimen>
        <Specimen label={caption.vault}>
          <div className={styles.column}>
            <VaultStatus vault={{ backend: "keychain" }} nodeCount={12} onUnlock={ignore} />
            <VaultStatus
              vault={{ backend: "file", locked: true, fileName: "infrastruttura.vault" }}
              nodeCount={12}
              onUnlock={ignore}
            />
            <VaultStatus
              vault={{ backend: "file", locked: false, fileName: "infrastruttura.vault" }}
              nodeCount={1}
              onUnlock={ignore}
            />
          </div>
        </Specimen>
      </div>
    </GallerySection>
  );
}
