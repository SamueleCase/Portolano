import { QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import { afterEach } from "vitest";

import { FeatureHost } from "../../app/FeatureHost";
import { featureRegistry } from "../../app/featureRegistry";
import { FeatureRegistryContext } from "../../app/features";
import { createQueryClient } from "../../ipc/queryClient";
import { closeAllRequests } from "../../state/requests";

// The requests of the window are a store of the module: a test leaves none
// open for the next one.
afterEach(closeAllRequests);

/**
 * For the tests of UI-WP5: renders `ui` as the window mounts the functions,
 * with the registrations of the app and the host of their requests, so that
 * `requestVault` and the other requests of the code open their dialogs.
 */
export function renderInWindow(ui: ReactElement) {
  const client = createQueryClient();
  const tree = (content: ReactElement) => (
    <QueryClientProvider client={client}>
      <FeatureRegistryContext value={featureRegistry}>
        {content}
        <FeatureHost />
      </FeatureRegistryContext>
    </QueryClientProvider>
  );
  const view = render(tree(ui));
  return { client, ...view, rerender: (next: ReactElement) => view.rerender(tree(next)) };
}
