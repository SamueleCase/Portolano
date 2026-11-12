import { useState } from "react";

import { RouteChain } from "../../components/RouteChain";
import { SessionRoute } from "../../components/SessionRoute";
import { Stepper } from "../../components/Stepper";
import { tabElementId } from "../../components/tabIds";
import { Tabs } from "../../components/Tabs";
import { gallery } from "../../i18n/gallery";
import { routeStops, steps, workTabs } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;

/** The work tabs, the steppers and the routes. */
export function NavigationSection() {
  const [tabs, setTabs] = useState(workTabs);
  const [selected, setSelected] = useState("s2");
  const [editorStep, setEditorStep] = useState(0);
  return (
    <GallerySection id="navigation" title={gallery.section.navigation}>
      <div className={styles.wide}>
        <Specimen label={caption.tabs}>
          <div className={styles.column}>
            <Tabs
              tabs={tabs}
              selectedId={selected}
              onSelect={setSelected}
              onClose={(id) => setTabs((current) => current.filter((tab) => tab.id !== id))}
              panelId="gallery-work"
            />
            <div
              id="gallery-work"
              role="tabpanel"
              aria-labelledby={tabElementId("gallery-work", selected)}
            />
          </div>
        </Specimen>
        <Specimen label={caption.stepperWizard}>
          <div className={styles.column}>
            <Stepper steps={steps.wizard} />
            <Stepper steps={steps.wizardSkipped} />
          </div>
        </Specimen>
        <Specimen label={caption.stepperEditor}>
          <div className={styles.column}>
            <Stepper
              variant="editor"
              onSelect={setEditorStep}
              steps={steps.editor.map((label, index) => ({
                label,
                state: index === editorStep ? "current" : index < editorStep ? "done" : "upcoming",
              }))}
            />
          </div>
        </Specimen>
      </div>
      <div className={styles.wide}>
        <Specimen label={caption.routeDetail}>
          <RouteChain variant="route" stops={routeStops.detail} />
        </Specimen>
        <Specimen label={caption.routeEditor}>
          <div className={styles.column}>
            <RouteChain variant="editor" stops={routeStops.editor} />
          </div>
        </Specimen>
        <Specimen label={caption.routeInherit}>
          <RouteChain variant="inherit" label={caption.routeInherit} stops={routeStops.inherit} />
        </Specimen>
        {(["connecting", "connected", "failed"] as const).map((phase) => (
          <Specimen key={phase} label={`${caption.sessionRoute} · ${caption.phases[phase]}`}>
            <div className={styles.column}>
              <SessionRoute stops={routeStops.session} phase={phase} />
            </div>
          </Specimen>
        ))}
      </div>
    </GallerySection>
  );
}
