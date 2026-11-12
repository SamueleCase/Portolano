import { useMutation, useQueryClient, type QueryKey } from "@tanstack/react-query";
import { useRef } from "react";

import { type IpcResult, unwrap } from "./result";

/** The data a successful change makes stale. */
export type Invalidates = readonly QueryKey[];

/** A command that changes something, with the keys it makes stale. */
export function useCommandMutation<A, T>(
  run: (args: A) => Promise<IpcResult<T>>,
  invalidates: Invalidates,
) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (args: A) => unwrap(await run(args)),
    onSuccess: () => invalidate(queryClient, invalidates),
  });
}

/**
 * A command whose arguments carry a secret typed by the user (SPEC-UI §1.2).
 *
 * TanStack Query keeps the variables of a mutation in its cache, where the
 * devtools, persisters and later renders can read them. Here the arguments
 * never become variables: `mutate` puts them in a ref, the mutation takes
 * them out and drops the ref before calling the backend, and the cache sees
 * `undefined`. The secret then lives only in the call to the backend.
 */
export function useSecretMutation<A, T>(
  run: (args: A) => Promise<IpcResult<T>>,
  invalidates: Invalidates,
) {
  const queryClient = useQueryClient();
  const pending = useRef<A | null>(null);
  const mutation = useMutation({
    mutationFn: async () => {
      const args = pending.current;
      pending.current = null;
      if (args === null) {
        throw new Error("secret mutation started without arguments");
      }
      return unwrap(await run(args));
    },
    onSuccess: () => invalidate(queryClient, invalidates),
  });
  return {
    /** Runs the command with `args`; they are not kept anywhere. */
    mutateAsync: (args: A): Promise<T> => {
      pending.current = args;
      return mutation.mutateAsync();
    },
    status: mutation.status,
    isPending: mutation.isPending,
    isError: mutation.isError,
    isSuccess: mutation.isSuccess,
    data: mutation.data,
    error: mutation.error,
    reset: mutation.reset,
  };
}

async function invalidate(
  queryClient: ReturnType<typeof useQueryClient>,
  invalidates: Invalidates,
): Promise<void> {
  await Promise.all(invalidates.map((queryKey) => queryClient.invalidateQueries({ queryKey })));
}
