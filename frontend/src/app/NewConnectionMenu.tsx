import { ContextMenu } from "../components/ContextMenu";
import { texts } from "../i18n/it";
import { useWindowActions } from "./windowActions";

interface NewConnectionMenuProps {
  /** The button that opened it: the menu opens under it. */
  anchor: HTMLElement;
  onClose: () => void;
}

/**
 * "Nuova connessione" (ambiguity 10): the protocol first, SSH or RDP; the editor
 * opens on the folder of the selection, which it can change.
 */
export function NewConnectionMenu({ anchor, onClose }: NewConnectionMenuProps) {
  const { create } = useWindowActions();
  const box = anchor.getBoundingClientRect();
  const { protocols } = texts.app.toolbar;
  return (
    <ContextMenu
      label={texts.app.toolbar.newConnectionMenu}
      entries={[
        { id: "ssh", label: protocols.ssh },
        { id: "rdp", label: protocols.rdp },
      ]}
      position={{ x: box.left, y: box.bottom }}
      onSelect={(id) => create(id === "rdp" ? "rdp" : "ssh")}
      onClose={onClose}
    />
  );
}
