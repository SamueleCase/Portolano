import type { NodeIconChoice } from "../../components/icons/NodeIcon";
import type { IconChoice, IconView } from "../../bindings";
import { iconChoice } from "../tree/treeModel";

/**
 * Between the icons as the backend names them (`IconChoice` to save,
 * `IconView` to show) and as the components draw them (`NodeIconChoice`,
 * with the file name the `portolano-icon://` scheme serves).
 */

type CustomIcon = Extract<IconView, { kind: "custom" }>;

/** The stored images of the document, with the file name the components draw each by. */
export function customImages(
  stored: readonly IconView[],
): { icon: CustomIcon; fileName: string }[] {
  return stored.flatMap((icon) => {
    const drawn = iconChoice(icon);
    return icon.kind === "custom" && drawn?.source === "custom"
      ? [{ icon, fileName: drawn.fileName }]
      : [];
  });
}

/** How the components draw the icon `choice`, with the stored images of the document. */
export function drawnIcon(
  choice: IconChoice | null,
  stored: readonly IconView[],
): NodeIconChoice | undefined {
  if (choice === null || choice.kind === "none") return undefined;
  if (choice.kind === "builtin") return { source: "builtin", name: choice.name };
  const image = stored.find((icon) => icon.kind === "custom" && icon.sha256 === choice.sha256);
  return image ? iconChoice(image) : undefined;
}

/** The icon to save for what the picker chose. */
export function savedIcon(
  choice: NodeIconChoice,
  stored: readonly IconView[],
): IconChoice | undefined {
  if (choice.source === "builtin") return { kind: "builtin", name: choice.name };
  const image = customImages(stored).find(({ fileName }) => fileName === choice.fileName)?.icon;
  return image ? { kind: "custom", sha256: image.sha256, mime: image.mime } : undefined;
}

/** The icon a node has, as the editor saves it again. */
export function iconChoiceOf(view: IconView | null): IconChoice {
  if (view === null) return { kind: "none" };
  return view.kind === "builtin"
    ? { kind: "builtin", name: view.name }
    : { kind: "custom", sha256: view.sha256, mime: view.mime };
}
