import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { StatusScreen } from "./StatusScreen";

/** The root of the interface. */
export function App({ queryClient }: { queryClient: QueryClient }) {
  return (
    <QueryClientProvider client={queryClient}>
      <StatusScreen />
    </QueryClientProvider>
  );
}
