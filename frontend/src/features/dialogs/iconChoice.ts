import type { NodeIconChoice } from "../../components/icons/NodeIcon";
import type { IconChoice, IconView } from "../../bindings";

/**
 * Between the icons as the backend names them (`IconChoice` to save,
 * `IconView` to show) and as the components draw them (`NodeIconChoice`,
 * with the file name the `portolano-icon://` scheme serves).
 */

/** The file name of a custom icon: the last segment of its URL (`<sha256>.<ext>`). */
export function iconFileName(icon: Extract<IconView, { kind: "custom" }>): string {
  return icon.url.split("/").pop() ?? "";
}

/** How the components draw the icon `choice`, with the stored images of the document. */
export function drawnIcon(
  choice: IconChoice | null,
  stored: readonly IconView[],
): NodeIconChoice | undefined {
  if (choice === null || choice.kind === "none") return undefined;
  if (choice.kind === "builtin") return { source: "builtin", name: choice.name };
  const image = stored.find(
    (icon): icon is Extract<IconView, { kind: "custom" }> =>
      icon.kind === "custom" && icon.sha256 === choice.sha256,
  );
  return image ? { source: "custom", fileName: iconFileName(image) } : undefined;
}

/** The icon to save for what the picker chose. */
export function savedIcon(
  choice: NodeIconChoice,
  stored: readonly IconView[],
): IconChoice | undefined {
  if (choice.source === "builtin") return { kind: "builtin", name: choice.name };
  const image = stored.find(
    (icon): icon is Extract<IconView, { kind: "custom" }> =>
      icon.kind === "custom" && iconFileName(icon) === choice.fileName,
  );
  return image ? { kind: "custom", sha256: image.sha256, mime: image.mime } : undefined;
}

/** The icon a node has, as the editor saves it again. */
export function iconChoiceOf(view: IconView | null): IconChoice {
  if (view === null) return { kind: "none" };
  return view.kind === "builtin"
    ? { kind: "builtin", name: view.name }
    : { kind: "custom", sha256: view.sha256, mime: view.mime };
}
