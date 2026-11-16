import { useQuery } from "@tanstack/react-query";

import { routeOf } from "../features/details/route";
import { SessionView } from "../features/sessions/SessionView";
import type { Session } from "../features/sessions/store";
import { nodeDetailsQuery } from "../ipc/queries";
import type { TreeEntry } from "../features/tree/treeModel";

interface SessionTabProps {
  session: Session;
  /** The node of the session, if it is still in the document. */
  entry: TreeEntry | undefined;
  /** The summary of the connection when the session opened. */
  summary: string;
}

/**
 * The body of a session tab: the view of its kind (`sessionViews` of UI-WP3),
 * with the summary of the connection and its route from the jump hosts.
 */
export function SessionTab({ session, entry, summary }: SessionTabProps) {
  const details = useQuery({ ...nodeDetailsQuery(session.nodeId), enabled: entry !== undefined });
  // Without jumps the route of the header is the destination alone.
  const host = details.data?.connection?.host;
  const route = routeOf(details.data) ?? (host === undefined ? undefined : [host]);
  return <SessionView session={session} summary={entry?.node.summary ?? summary} route={route} />;
}
