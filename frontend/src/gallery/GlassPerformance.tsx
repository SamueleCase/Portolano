import { GlassPanel } from "../components/GlassPanel";
import { BackgroundCanvas } from "../design/background/BackgroundCanvas";
import { gallery } from "../i18n/gallery";
import styles from "./GlassPerformance.module.css";
import { GALLERY_SEED } from "./galleryOptions";

/**
 * The page of the glass measurement (DEC-028.6, `npm run perf:glass`): the
 * animated background under three glass panels laid out like the main
 * window, with the blur of the tokens.
 */
export function GlassPerformance() {
  return (
    <>
      <BackgroundCanvas seed={GALLERY_SEED} />
      <div className={styles.window}>
        <GlassPanel as="header" className={styles.toolbar}>
          {gallery.glass.toolbar}
        </GlassPanel>
        <GlassPanel as="aside" className={styles.panel}>
          {gallery.glass.sidebar}
        </GlassPanel>
        <GlassPanel as="main" className={styles.panel}>
          <h1 className={styles.title}>{gallery.glass.title}</h1>
          {gallery.glass.detail}
        </GlassPanel>
      </div>
    </>
  );
}
