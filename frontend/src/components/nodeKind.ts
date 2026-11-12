/** The kinds of node the interface draws (inventario §6). */
export type NodeKind = "folder" | "ssh" | "rdp" | "credential" | "unsupported";

/** Whether a node of `kind` can be connected to. */
export function isConnection(kind: NodeKind): kind is "ssh" | "rdp" {
  return kind === "ssh" || kind === "rdp";
}
