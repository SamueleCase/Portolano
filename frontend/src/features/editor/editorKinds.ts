import { texts } from "../../i18n/it";
import type { EditorKind } from "./editorValues";

/** The label of step 2, which depends on the kind (inventario §4.25). */
export function secondStepLabel(kind: EditorKind): string {
  const steps = texts.editor.steps;
  switch (kind) {
    case "ssh":
    case "rdp":
      return steps.connection;
    case "credential":
      return steps.access;
    case "folder":
      return steps.folderCredential;
  }
}

/** The default port of a protocol, the placeholder of an empty port. */
export const DEFAULT_PORTS: Readonly<Record<"ssh" | "rdp", number>> = { ssh: 22, rdp: 3389 };
