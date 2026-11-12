import { useMemo, useState } from "react";

import { Terminal } from "../../components/terminal/Terminal";
import { backgroundControl } from "../../design/background/backgroundControl";
import { gallery } from "../../i18n/gallery";
import { echoTerminal } from "../echoTerminal";
import { sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

/** The terminal on a fake session that echoes (ambiguity 24). */
export function TerminalSection() {
  const io = useMemo(() => echoTerminal(sample.terminalTitle, sample.link), []);
  const [title, setTitle] = useState("");
  const [opened, setOpened] = useState("");
  return (
    <GallerySection id="terminal" title={gallery.section.terminal}>
      <Specimen label={`${gallery.caption.terminalTitle}: ${title}`}>
        <div className={styles.column}>
          <div className={styles.terminalBox}>
            <Terminal
              io={io}
              onTitle={setTitle}
              onOpenLink={setOpened}
              onActivity={() => backgroundControl.noteSessionActivity()}
            />
          </div>
          <span className={styles.caption} data-testid="opened-link">
            {opened}
          </span>
        </div>
      </Specimen>
    </GallerySection>
  );
}
