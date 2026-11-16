import type { ClosableProps } from "../../app/features";
import { PreferencesDialog } from "./PreferencesDialog";

/** The preferences at the hook point `preferences` of the window, applied at once. */
export function PreferencesFeature({ onClose }: ClosableProps) {
  return <PreferencesDialog open onClose={onClose} />;
}
