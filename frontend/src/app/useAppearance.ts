import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";

import { applyAppearance } from "../design/appearance";
import { backgroundControl } from "../design/background/backgroundControl";
import { uiPreferencesQuery } from "../ipc/queries";

/**
 * Applies the preferences of the interface (`ui_prefs_get`): density and
 * reduce motion to the whole window, the animated background to its canvas
 * (SPEC-UI §5.2). They change at once when the preferences change.
 */
export function useAppearance(): void {
  const preferences = useQuery(uiPreferencesQuery).data;
  useEffect(() => {
    if (!preferences) return;
    const { density, reduceMotion, background } = preferences;
    applyAppearance(document.documentElement, { density, reduceMotion });
    backgroundControl.setPreferences({ mode: background, reduceMotion });
  }, [preferences]);
}
