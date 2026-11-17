import { useQuery } from "@tanstack/react-query";

import type { VaultBackend } from "../../components/SecretBadge";
import type { VaultKind } from "../../bindings";
import { vaultStatusQuery } from "../../ipc/queries";

/** How the badges name the vault: the in-memory vault of the tests reads as the file vault. */
export function vaultBackend(kind: VaultKind | undefined): VaultBackend {
  return kind === "keychain" ? "keychain" : "file";
}

/** The vault of the configuration, for the badges and the texts. */
export function useVaultState(): { backend: VaultBackend; locked: boolean } {
  const { data } = useQuery(vaultStatusQuery);
  return { backend: vaultBackend(data?.kind), locked: data?.locked ?? false };
}
