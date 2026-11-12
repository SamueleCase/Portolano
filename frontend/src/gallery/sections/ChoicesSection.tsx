import { useState } from "react";

import { CredentialField } from "../../components/CredentialField";
import { FavoriteToggle } from "../../components/FavoriteToggle";
import { FilterChip } from "../../components/FilterChip";
import { SecretRow } from "../../components/SecretRow";
import { SegmentedControl } from "../../components/SegmentedControl";
import { Toggle } from "../../components/Toggle";
import { ToggleRow } from "../../components/ToggleRow";
import { gallery } from "../../i18n/gallery";
import { texts } from "../../i18n/it";
import { sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;
const ignore = () => undefined;

/** Toggles, segmented controls, filters, the favorite pill, credentials and secrets. */
export function ChoicesSection() {
  const [toggles, setToggles] = useState([true, false, true, false]);
  const [resolution, setResolution] = useState<string>(sample.resolutions[0]);
  const [theme, setTheme] = useState<string>(sample.themes[0]);
  const [filter, setFilter] = useState<string>(sample.filters[0]);
  const [favorites, setFavorites] = useState(false);
  const [report, setReport] = useState<string>(sample.reportFilters[1]);
  const [favorite, setFavorite] = useState(true);
  return (
    <GallerySection id="choices" title={gallery.section.choices}>
      <div className={styles.grid}>
        <Specimen label={caption.toggles}>
          <Toggle label={caption.toggleOn} checked onChange={ignore} />
          <Toggle label={caption.toggleOff} checked={false} onChange={ignore} />
          <Toggle label={gallery.states.disabled} checked={false} disabled onChange={ignore} />
        </Specimen>
        <Specimen label={caption.toggleRows}>
          <div className={styles.column}>
            {sample.toggles.map((label, index) => (
              <ToggleRow
                key={label}
                label={label}
                description={index === 3 ? sample.x11Hint : undefined}
                checked={toggles[index] ?? false}
                disabled={index === 3 && !toggles[2]}
                onChange={(checked) =>
                  setToggles((current) => current.map((on, at) => (at === index ? checked : on)))
                }
              />
            ))}
          </div>
        </Specimen>
        <Specimen label={caption.filters}>
          {sample.filters.map((label) => (
            <FilterChip key={label} active={filter === label} onToggle={() => setFilter(label)}>
              {label}
            </FilterChip>
          ))}
          <FilterChip
            active={favorites}
            onToggle={() => setFavorites(!favorites)}
            label={sample.favorites}
          >
            ★
          </FilterChip>
        </Specimen>
        <Specimen label={caption.reportFilters}>
          {sample.reportFilters.map((label) => (
            <FilterChip
              key={label}
              variant="report"
              active={report === label}
              onToggle={() => setReport(label)}
            >
              {label}
            </FilterChip>
          ))}
        </Specimen>
        <Specimen label={caption.favorite}>
          <FavoriteToggle favorite={favorite} onChange={setFavorite} />
          <FavoriteToggle favorite={false} onChange={ignore} />
        </Specimen>
      </div>
      <div className={styles.wide}>
        <Specimen label={caption.segmented}>
          <SegmentedControl
            label={sample.resolution}
            value={resolution}
            onChange={setResolution}
            options={sample.resolutions.map((value) => ({ value, label: value }))}
          />
          <SegmentedControl
            label={sample.theme}
            value={theme}
            onChange={setTheme}
            options={sample.themes.map((value, index) => ({
              value,
              label: value,
              unavailable: index === 1,
            }))}
          />
        </Specimen>
        <Specimen label={caption.credentialInherited}>
          <CredentialField
            label={caption.credential}
            mode={texts.components.credentialModes.inherit.title}
            origin={sample.inheritedOrigin}
            password="present"
            backend="keychain"
            onChange={ignore}
          />
        </Specimen>
        <Specimen label={caption.credentialMissing}>
          <CredentialField
            label={caption.credentialGateway}
            mode={sample.fromCredential}
            origin={sample.credentialName}
            password="absent"
            backend="keychain"
            onChange={ignore}
            onShowCredential={ignore}
          />
        </Specimen>
        <Specimen label={caption.credentialRdp}>
          <CredentialField
            label={caption.credential}
            mode={texts.components.credentialModes.inline.title}
            user={sample.domainUser}
            password="locked"
            backend="file"
            onChange={ignore}
          />
        </Specimen>
      </div>
      <div className={styles.wide}>
        <SecretRow
          label={sample.password}
          state="present"
          backend="keychain"
          onSet={ignore}
          onClear={ignore}
        />
        <SecretRow label={sample.passphrase} state="absent" backend="keychain" onSet={ignore} />
        <SecretRow
          label={sample.token}
          state="staged"
          backend="file"
          onSet={ignore}
          onClear={ignore}
        />
        <SecretRow label={sample.password} state="locked" backend="file" onSet={ignore} />
      </div>
    </GallerySection>
  );
}
