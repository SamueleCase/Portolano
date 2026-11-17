import type { EditorProps } from "../../app/features";
import { NodeEditor } from "./NodeEditor";

/**
 * The editor at the hook point `editor` of the window: it opens on the
 * target of the request; once saved, the window selects the node and the
 * editor closes.
 */
export function EditorFeature({ target, onClose, onSaved }: EditorProps) {
  return (
    <NodeEditor
      request={target}
      onClose={onClose}
      onSaved={(node) => {
        onSaved(node.id);
        onClose();
      }}
    />
  );
}
