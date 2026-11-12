import { QueryClient } from "@tanstack/react-query";

/**
 * The query client of the app (DEC-031.5). The backend is a local process,
 * not the network: queries and mutations run whatever the browser thinks of
 * the connection (`networkMode: "always"`, otherwise WebKit going "offline"
 * would pause them), and a failed command is not repeated, since the same
 * command would fail the same way and the error has to reach the interface.
 */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { networkMode: "always", retry: false },
      mutations: { networkMode: "always", retry: false },
    },
  });
}
