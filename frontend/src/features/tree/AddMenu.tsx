import { Plus } from "lucide-react";
import { useState } from "react";

import { GuardedButton } from "../../app/GuardedButton";
import { NewConnectionMenu } from "../../app/NewConnectionMenu";
import { useWindowActions } from "../../app/windowActions";
import { classNames } from "../../components/classNames";
import { ContextMenu, type MenuEntry } from "../../components/ContextMenu";
import iconStyles from "../../components/icons/Icon.module.css";
import { iconSizeClass } from "../../components/icons/iconSizes";
import { texts } from "../../i18n/it";
import { useWorkspace } from "../../state/workspace";
import styles from "./Sidebar.module.css";

/**
 * The "+" next to the search (DEC-050): a menu with the new connection (the
 * protocol first, ambiguity 10), the new folder, the new credential and the
 * import. An entry whose function is not mounted is disabled with the reason
 * (DEC-035); with none of them mounted the button itself is.
 *
 * The protocol menu opens under "+" also from ⌘N and the native menu
 * (`connectionMenuOpen`). Both menus give the focus back to what had it
 * before them, so Esc returns to "+".
 */
export function AddMenu() {
  const t = texts.tree.add;
  const { reasons, shortcuts } = texts.app;
  const actions = useWindowActions();
  const [anchor, setAnchor] = useState<HTMLButtonElement | null>(null);
  const [open, setOpen] = useState(false);
  const protocolMenuOpen = useWorkspace((state) => state.connectionMenuOpen);
  const setProtocolMenu = useWorkspace((state) => state.setConnectionMenu);
  const creates = actions.offers("editor");
  const imports = actions.offers("import");
  const offered = (available: boolean) =>
    available ? { disabled: false } : { disabled: true, reason: reasons.unavailable };
  const entries: MenuEntry[] = [
    {
      id: "connection",
      label: t.newConnection,
      shortcut: shortcuts.newConnection,
      ...offered(creates),
    },
    { id: "folder", label: t.newFolder, shortcut: shortcuts.newFolder, ...offered(creates) },
    { id: "credential", label: t.newCredential, ...offered(creates) },
    { separator: true },
    { id: "import", label: t.import, shortcut: shortcuts.import, ...offered(imports) },
  ];
  const choose = (id: string) => {
    if (id === "connection") setProtocolMenu(true);
    else if (id === "folder") actions.create("folder");
    else if (id === "credential") actions.create("credential");
    else actions.openImport();
  };
  const box = open ? anchor?.getBoundingClientRect() : undefined;

  return (
    <>
      <GuardedButton
        ref={setAnchor}
        className={styles.addButton}
        aria-label={t.label}
        aria-haspopup="menu"
        aria-expanded={open || protocolMenuOpen}
        reason={creates || imports ? undefined : reasons.unavailable}
        hint={t.label}
        onClick={() => setOpen(true)}
      >
        <Plus aria-hidden="true" className={classNames(iconStyles.icon, iconSizeClass.tree)} />
      </GuardedButton>
      {/* The protocol menu takes the place of this one. */}
      {open && !protocolMenuOpen && box && (
        <ContextMenu
          label={t.label}
          entries={entries}
          position={{ x: box.left, y: box.bottom }}
          onSelect={choose}
          onClose={() => setOpen(false)}
        />
      )}
      {protocolMenuOpen && anchor && (
        <NewConnectionMenu anchor={anchor} onClose={() => setProtocolMenu(false)} />
      )}
    </>
  );
}
