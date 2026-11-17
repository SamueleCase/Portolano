import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { ToggleRow } from "../../components/ToggleRow";
import type { InterfacePreferences_Deserialize } from "../../bindings";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useUiPrefsSet } from "../../ipc/mutations";
import { uiPreferencesQuery } from "../../ipc/queries";
import { ChoiceSetting } from "./ChoiceSetting";
import styles from "./Preferences.module.css";

/**
 * Density, animated background and reduce motion (SPEC-UI §5.2), saved in
 * `ui.toml` with `ui_prefs_set`; the frame of the app applies them.
 */
export function AppearanceSettings({ enabled }: { enabled: boolean }) {
  const t = texts.preferences.appearance;
  const { data: preferences } = useQuery({ ...uiPreferencesQuery, enabled });
  const save = useUiPrefsSet();
  const [error, setError] = useState<string>();
  if (!preferences) return null;

  const change = (part: InterfacePreferences_Deserialize) => {
    setError(undefined);
    save
      .mutateAsync({ ...preferences, ...part })
      .catch((failure: unknown) => setError(presentFailure(failure).message));
  };

  return (
    <>
      <ChoiceSetting
        label={t.density}
        options={[
          { value: "normal", label: t.densities.normal },
          { value: "compact", label: t.densities.compact },
        ]}
        value={preferences.density}
        onChange={(density) => change({ density })}
      />
      <ChoiceSetting
        label={t.background}
        options={[
          { value: "on", label: t.backgrounds.on },
          { value: "reduced", label: t.backgrounds.reduced },
          { value: "off", label: t.backgrounds.off },
        ]}
        value={preferences.background}
        onChange={(background) => change({ background })}
      />
      <ToggleRow
        label={t.reduceMotion}
        description={t.reduceMotionHint}
        checked={preferences.reduceMotion}
        onChange={(reduceMotion) => change({ reduceMotion })}
      />
      {error && (
        <span className={styles.error} role="alert">
          {error}
        </span>
      )}
    </>
  );
}
