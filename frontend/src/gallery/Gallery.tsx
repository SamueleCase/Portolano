import { useEffect, useState } from "react";

import { GlassPanel } from "../components/GlassPanel";
import { Logo } from "../components/Logo";
import { SegmentedControl } from "../components/SegmentedControl";
import { ToggleRow } from "../components/ToggleRow";
import { applyAppearance, type Density } from "../design/appearance";
import { BackgroundCanvas } from "../design/background/BackgroundCanvas";
import { backgroundControl, type BackgroundMode } from "../design/background/backgroundControl";
import { gallery } from "../i18n/gallery";
import styles from "./Gallery.module.css";
import { GALLERY_SEED, type GalleryOptions } from "./galleryOptions";
import { BadgesSection } from "./sections/BadgesSection";
import { ButtonsSection } from "./sections/ButtonsSection";
import { ChoicesSection } from "./sections/ChoicesSection";
import { FieldsSection } from "./sections/FieldsSection";
import { FoundationsSection } from "./sections/FoundationsSection";
import { ListsSection } from "./sections/ListsSection";
import { NavigationSection } from "./sections/NavigationSection";
import { OverlaysSection } from "./sections/OverlaysSection";
import { SelectorsSection } from "./sections/SelectorsSection";
import { SessionsSection } from "./sections/SessionsSection";
import { SurfacesSection } from "./sections/SurfacesSection";
import { TerminalSection } from "./sections/TerminalSection";
import { TreeSection } from "./sections/TreeSection";
import { UntrustedSection } from "./sections/UntrustedSection";

const sections = [
  ["foundations", FoundationsSection],
  ["buttons", ButtonsSection],
  ["fields", FieldsSection],
  ["choices", ChoicesSection],
  ["badges", BadgesSection],
  ["navigation", NavigationSection],
  ["tree", TreeSection],
  ["lists", ListsSection],
  ["surfaces", SurfacesSection],
  ["overlays", OverlaysSection],
  ["selectors", SelectorsSection],
  ["terminal", TerminalSection],
  ["untrusted", UntrustedSection],
  ["sessions", SessionsSection],
] as const;

/**
 * The gallery of the components (SPEC-UI §5.2), over the animated
 * background, with the density, reduce motion and background preferences.
 * Only in development and e2e builds (`main.tsx`).
 */
export function Gallery({ options }: { options: GalleryOptions }) {
  const [density, setDensity] = useState<Density>(options.density);
  const [reduceMotion, setReduceMotion] = useState(options.reduceMotion);
  const [background, setBackground] = useState<BackgroundMode>(options.background);
  const { controls } = gallery;

  useEffect(() => {
    applyAppearance(document.documentElement, { density, reduceMotion });
    backgroundControl.setPreferences({ mode: background, reduceMotion });
  }, [density, reduceMotion, background]);

  return (
    <>
      <BackgroundCanvas seed={GALLERY_SEED} />
      <div className={styles.page}>
        <GlassPanel as="header" className={styles.header}>
          <Logo />
          <h1 className={styles.title}>{gallery.title}</h1>
          <div className={styles.controls}>
            <SegmentedControl
              label={controls.density}
              value={density}
              onChange={setDensity}
              options={[
                { value: "normal", label: controls.densities.normal },
                { value: "compact", label: controls.densities.compact },
              ]}
            />
            <SegmentedControl
              label={controls.background}
              value={background}
              onChange={setBackground}
              options={[
                { value: "on", label: controls.backgrounds.on },
                { value: "reduced", label: controls.backgrounds.reduced },
                { value: "off", label: controls.backgrounds.off },
              ]}
            />
            <ToggleRow
              label={controls.reduceMotion}
              checked={reduceMotion}
              onChange={setReduceMotion}
            />
          </div>
        </GlassPanel>
        <GlassPanel as="aside" className={styles.nav}>
          <nav aria-label={gallery.sections}>
            <ul className={styles.navList}>
              {sections.map(([id]) => (
                <li key={id}>
                  <a href={`#${id}`}>{gallery.section[id]}</a>
                </li>
              ))}
            </ul>
          </nav>
        </GlassPanel>
        <main className={styles.sections}>
          {sections.map(([id, Section]) => (
            <Section key={id} />
          ))}
        </main>
      </div>
    </>
  );
}
