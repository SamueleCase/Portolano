import { useQueries } from "@tanstack/react-query";

import type { JsonValue, NodeDetailsView_Serialize } from "../../bindings";
import { texts } from "../../i18n/it";
import { nodeDetailsQuery } from "../../ipc/queries";

/** A jump host of an SSH connection, as its settings store it. */
export type Jump = { ref: string } | { address: string };

/** The jump hosts in `settings`, in order: `{ "$ref": id }` or `{ "address": "…" }`. */
export function jumpsOf(settings: { [key: string]: JsonValue }): Jump[] {
  const jumps = settings["jump_hosts"];
  if (!Array.isArray(jumps)) return [];
  return jumps.flatMap((jump): Jump[] => {
    if (jump === null || typeof jump !== "object" || Array.isArray(jump)) return [];
    const ref = jump["$ref"];
    if (typeof ref === "string") return [{ ref }];
    const address = jump["address"];
    return typeof address === "string" ? [{ address }] : [];
  });
}

/**
 * The host of an address `[user@]host[:port]`, as the route shows it
 * (inventario §4.21: only the host of an address).
 */
export function addressHost(address: string): string {
  const host = address.slice(address.lastIndexOf("@") + 1);
  if (host.startsWith("[")) {
    const end = host.indexOf("]");
    return end === -1 ? host : host.slice(1, end);
  }
  const colon = host.indexOf(":");
  return colon === -1 ? host : host.slice(0, colon);
}

/**
 * The stops of the route of connection `details` after "tu": the hosts of its
 * jumps and its own host; `undefined` without jumps or for another node.
 * The hosts of the jumps of the document come from their detail.
 */
export function useRoute(details: NodeDetailsView_Serialize | undefined): string[] | undefined {
  const jumps = details?.connection ? jumpsOf(details.connection.settings) : [];
  const refs = jumps.flatMap((jump) => ("ref" in jump ? [jump.ref] : []));
  const hosts = useQueries({
    queries: refs.map((id) => ({
      ...nodeDetailsQuery(id),
      select: (jump: NodeDetailsView_Serialize) => jump.connection?.host ?? jump.name,
    })),
  });
  if (details?.connection == null || jumps.length === 0) return undefined;
  // A reference to a node that is gone shows as a dash (validation reports it).
  const byRef = new Map(refs.map((id, position) => [id, hosts[position]?.data]));
  const labels = jumps.map((jump) =>
    "address" in jump
      ? addressHost(jump.address)
      : (byRef.get(jump.ref) ?? texts.details.credential.nothing),
  );
  return [...labels, details.connection.host];
}
