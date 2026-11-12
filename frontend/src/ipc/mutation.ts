import { type QueryKey, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useMemo, useRef } from "react";

import { type IpcResult, unwrap } from "./result";

/** The data a successful change makes stale. */
export type Invalidates = readonly QueryKey[];

/**
 * A command that changes something, with the keys it makes stale. A command
 * without arguments is called with `mutate()` (`A` defaults to `void`).
 */
export function useCommandMutation<A = void, T = unknown>(
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
 * devtools, persisters and later renders can read them. Here the variables
 * are only a counter: each call puts its arguments in a map under a new
 * token, and the mutation takes them out before calling the backend, on
 * success and on failure alike. Two calls in a row each keep their own
 * arguments; the secret lives only in the call to the backend.
 */
export function useSecretMutation<A, T>(
  run: (args: A) => Promise<IpcResult<T>>,
  invalidates: Invalidates,
) {
  const queryClient = useQueryClient();
  const pending = useRef(new Map<number, A>());
  const nextToken = useRef(0);
  const mutation = useMutation({
    mutationFn: async (token: number) => {
      const args = pending.current.get(token);
      pending.current.delete(token);
      if (args === undefined) {
        throw new Error("secret mutation started without arguments");
      }
      return unwrap(await run(args));
    },
    onSuccess: () => invalidate(queryClient, invalidates),
  });
  const { mutateAsync: start } = mutation;
  const mutateAsync = useCallback(
    (args: A): Promise<T> => {
      nextToken.current += 1;
      const token = nextToken.current;
      pending.current.set(token, args);
      // The map is emptied even if the mutation never runs its function.
      return start(token).finally(() => pending.current.delete(token));
    },
    [start],
  );
  /** Whether arguments of a call are still held: never once the calls are over. */
  const holdsArguments = useCallback(() => pending.current.size > 0, []);
  const { status, isPending, isError, isSuccess, data, error, reset } = mutation;
  return useMemo(
    () => ({
      mutateAsync,
      holdsArguments,
      status,
      isPending,
      isError,
      isSuccess,
      data,
      error,
      reset,
    }),
    [mutateAsync, holdsArguments, status, isPending, isError, isSuccess, data, error, reset],
  );
}

async function invalidate(
  queryClient: ReturnType<typeof useQueryClient>,
  invalidates: Invalidates,
): Promise<void> {
  await Promise.all(invalidates.map((queryKey) => queryClient.invalidateQueries({ queryKey })));
}
