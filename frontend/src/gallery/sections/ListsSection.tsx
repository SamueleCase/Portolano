import { useState } from "react";

import { Button } from "../../components/Button";
import { NodeTypeIcon } from "../../components/icons/NodeTypeIcon";
import { KeyValueGrid } from "../../components/KeyValueGrid";
import { ReportRow } from "../../components/ReportRow";
import { SortableList } from "../../components/SortableList";
import { SortableRow } from "../../components/SortableRow";
import { SourceChip } from "../../components/SourceChip";
import { ValidationRow } from "../../components/ValidationRow";
import { gallery } from "../../i18n/gallery";
import {
  type JumpHost,
  jumpHosts,
  reportRows,
  sample,
  sessionFacts,
  validationRows,
} from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;
const ignore = () => undefined;

function JumpContent({ jump }: { jump: JumpHost }) {
  return (
    <>
      {jump.source === "tree" && <NodeTypeIcon kind="ssh" size="list" />}
      <span>{jump.name}</span>
      {jump.detail && <span className={styles.jumpPath}>{jump.detail}</span>}
      <SourceChip source={jump.source} />
    </>
  );
}

/** The sortable jump list, the validation and report rows, a key-value grid. */
export function ListsSection() {
  const [jumps, setJumps] = useState(jumpHosts);
  const lifted = jumpHosts[0];
  return (
    <GallerySection id="lists" title={gallery.section.lists}>
      <div className={styles.wide}>
        <Specimen label={caption.sortable}>
          <div className={styles.column}>
            <SortableList
              label={caption.sortable}
              items={jumps}
              itemKey={(jump) => jump.id}
              itemName={(jump) => jump.name}
              renderItem={(jump) => <JumpContent jump={jump} />}
              onMove={(from, to) =>
                setJumps((current) => {
                  const next = [...current];
                  const [moved] = next.splice(from, 1);
                  if (moved) next.splice(to, 0, moved);
                  return next;
                })
              }
              onRemove={(index) => setJumps((current) => current.filter((_, at) => at !== index))}
              footer={
                <>
                  <Button variant="add">{sample.addFromTree}</Button>
                  <Button variant="add">{sample.addAddress}</Button>
                </>
              }
            />
          </div>
        </Specimen>
        {lifted && (
          <Specimen label={gallery.states.lifted}>
            <ol className={styles.column}>
              <SortableRow
                position={1}
                name={lifted.name}
                lifted
                instructionsId=""
                onRemove={ignore}
              >
                <JumpContent jump={lifted} />
              </SortableRow>
            </ol>
          </Specimen>
        )}
        <Specimen label={caption.validationRows}>
          <ul className={styles.column}>
            {validationRows.map((row) => (
              <ValidationRow key={row.path} {...row} onGoToNode={ignore} />
            ))}
          </ul>
        </Specimen>
        <Specimen label={caption.reportRows}>
          <ul className={styles.column}>
            {reportRows.map((row) => (
              <ReportRow
                key={row.path}
                {...row}
                onGoToNode={row.outcome === "skipped" ? undefined : ignore}
              />
            ))}
          </ul>
        </Specimen>
        <Specimen label={caption.keyValueGrid}>
          <KeyValueGrid width="session" rows={sessionFacts} />
        </Specimen>
      </div>
    </GallerySection>
  );
}
