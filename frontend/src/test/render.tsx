import { QueryClientProvider } from "@tanstack/react-query";
import { render, type RenderResult } from "@testing-library/react";
import type { ReactElement } from "react";

import { createQueryClient } from "../ipc/queryClient";

/** Renders `ui` with a fresh query cache configured as the app's (DEC-031.5). */
export function renderWithQueries(ui: ReactElement): RenderResult {
  const client = createQueryClient();
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}
