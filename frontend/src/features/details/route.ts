import type { NodeDetailsView_Serialize } from "../../bindings";
import { texts } from "../../i18n/it";

/**
 * The stops of the route of connection `details` after "tu" (inventario
 * §4.21): the hosts of its jumps, as the backend gives them (DEC-041.5), and
 * its own host; `undefined` without jumps. A jump to a node that is gone shows
 * as a dash (the validation reports it).
 */
export function routeOf(details: NodeDetailsView_Serialize | undefined): string[] | undefined {
  const connection = details?.connection;
  if (!connection || connection.jumps.length === 0) return undefined;
  return [
    ...connection.jumps.map((host) => host ?? texts.details.credential.nothing),
    connection.host,
  ];
}
