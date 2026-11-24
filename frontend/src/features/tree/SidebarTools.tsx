import { ListChecks, type LucideIcon, Settings } from "lucide-react";

import { GuardedButton } from "../../app/GuardedButton";
import { type Feature } from "../../app/features";
import { useWindowActions } from "../../app/windowActions";
import { classNames } from "../../components/classNames";
import iconStyles from "../../components/icons/Icon.module.css";
import { iconSizeClass } from "../../components/icons/iconSizes";
import { texts } from "../../i18n/it";
import styles from "./Sidebar.module.css";

interface ToolProps {
  feature: Feature;
  label: string;
  shortcut: string;
  icon: LucideIcon;
  onClick: () => void;
}

/** A button of the footer: an icon, named by its label and its tooltip. */
function Tool({ feature, label, shortcut, icon: Glyph, onClick }: ToolProps) {
  const actions = useWindowActions();
  return (
    <GuardedButton
      variant="ghost"
      className={styles.toolButton}
      aria-label={label}
      reason={actions.offers(feature) ? undefined : texts.app.reasons.unavailable}
      hint={label}
      shortcut={shortcut}
      onClick={() => onClick()}
    >
      <Glyph aria-hidden="true" className={classNames(iconStyles.icon, iconSizeClass.tree)} />
    </GuardedButton>
  );
}

/**
 * The tools on the right of the footer of the sidebar (DEC-050): "Valida"
 * and "Preferenze", as icons; one whose function is not mounted is disabled
 * with the reason (DEC-035).
 */
export function SidebarTools() {
  const t = texts.tree.tools;
  const { shortcuts } = texts.app;
  const actions = useWindowActions();
  return (
    <div role="group" aria-label={t.label} className={styles.tools}>
      <Tool
        feature="validation"
        label={t.validate}
        shortcut={shortcuts.validate}
        icon={ListChecks}
        onClick={actions.openValidation}
      />
      <Tool
        feature="preferences"
        label={t.preferences}
        shortcut={shortcuts.preferences}
        icon={Settings}
        onClick={actions.openPreferences}
      />
    </div>
  );
}
