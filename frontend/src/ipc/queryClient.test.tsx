import { QueryClientProvider, onlineManager, useQuery } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it } from "vitest";

import { installFakeBackend } from "../test/fakeBackend";
import { vaultStatusQuery } from "./queries";
import { createQueryClient } from "./queryClient";

afterEach(() => onlineManager.setOnline(true));

describe("createQueryClient", () => {
  it("asks the backend even when the browser thinks it is offline", async () => {
    installFakeBackend();
    onlineManager.setOnline(false);
    const client = createQueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useQuery(vaultStatusQuery), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.isPaused).toBe(false);
  });

  it("does not repeat a failed command", () => {
    const options = createQueryClient().getDefaultOptions();

    expect(options.queries?.retry).toBe(false);
    expect(options.mutations?.retry).toBe(false);
  });
});
