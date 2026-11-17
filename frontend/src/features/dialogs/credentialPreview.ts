import type {
  CredentialPreviewData,
  CredentialMode,
  ResolvedCredential,
} from "../../components/credentialPreviewData";
import type { SecretState, VaultBackend } from "../../components/SecretBadge";
import type {
  CredentialChoice_Deserialize,
  CredentialView_Serialize,
  NodeDetailsView_Serialize,
  SecretState as StoredSecretState,
} from "../../bindings";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { pathName, readablePath } from "../tree/treeModel";
import { type IndexedNode, credentialsNamed } from "./treeIndex";

/**
 * "Da dove arriverà" for a choice not saved yet, from what the backend
 * already says: the trace of the folder (Eredita), the credential node (Da
 * una credenziale), the credentials of the tree (Per nome). It is a preview
 * only: the backend resolves the credential at each connection.
 */

/** The selector's mode of a choice of the backend, and back. */
export const modeOfChoice: Record<CredentialChoice_Deserialize["mode"], CredentialMode> = {
  inherit: "inherit",
  none: "none",
  prompt: "ask",
  inline: "inline",
  byId: "linked",
  byName: "byName",
};

/** The state of a stored secret as the badges show it; `undefined` when it is not set. */
export function badgeState(state: StoredSecretState, locked: boolean): SecretState | undefined {
  if (state === "not_set") return undefined;
  if (locked) return "locked";
  return state;
}

/** User, domain and the states of the secrets of a resolved credential. */
export function resolvedFacts(
  credential: CredentialView_Serialize,
  locked: boolean,
  backend: VaultBackend,
): ResolvedCredential {
  const t = texts.dialogs.credential.key;
  const key = credential.private_key;
  const keyText =
    key === null
      ? undefined
      : key.kind === "file"
        ? interpolate(t.file, { path: key.path })
        : t.vault[backend];
  return {
    user: credential.username ?? undefined,
    domain: credential.domain ?? undefined,
    password: badgeState(credential.password, locked),
    key:
      keyText && credential.passphrase !== "not_set"
        ? interpolate(t.withPassphrase, { key: keyText })
        : keyText,
  };
}

function factsOf(details: NodeDetailsView_Serialize | undefined, backend: VaultBackend) {
  return details?.effective?.kind === "resolved"
    ? resolvedFacts(details.effective.credential, details.vaultLocked, backend)
    : {};
}

/**
 * What a node named `nodeName` inherits from `parent` (`null` when it sits
 * at the root, `undefined` while its detail loads).
 */
export function inheritPreview(
  nodeName: string,
  parent: NodeDetailsView_Serialize | null | undefined,
  backend: VaultBackend,
): CredentialPreviewData {
  const end = parent?.trace?.end;
  if (!parent || !parent.trace || !end || end.kind === "root") {
    return { mode: "inherit", node: nodeName, source: null };
  }
  const chain = [nodeName, ...parent.trace.steps.map((step) => pathName(step.path))];
  const t = texts.dialogs.credential.chain;
  if (end.kind === "credential") chain.push(pathName(end.node.path));
  if (end.kind === "prompt") chain.push(t.prompt);
  if (end.kind === "none") chain.push(t.none);
  return {
    mode: "inherit",
    node: nodeName,
    source: { chain, credential: factsOf(parent, backend) },
  };
}

/** The credential node a choice "Da una credenziale" points to. */
export function linkedPreview(
  credential: NodeDetailsView_Serialize | undefined,
  backend: VaultBackend,
): CredentialPreviewData {
  return {
    mode: "linked",
    credential: credential
      ? {
          name: credential.name,
          path: readablePath(credential.path),
          resolved: factsOf(credential, backend),
        }
      : null,
  };
}

/**
 * What a name matches today (SPEC §4.3): one credential, none, or several.
 * `found` is the detail of the only match, once loaded.
 */
export function byNamePreview(
  name: string,
  entries: readonly IndexedNode[],
  found: NodeDetailsView_Serialize | undefined,
  backend: VaultBackend,
): CredentialPreviewData {
  const matches = credentialsNamed(entries, name);
  const [only] = matches;
  if (!only) return { mode: "byName", name, match: { kind: "notFound" } };
  if (matches.length > 1) {
    return {
      mode: "byName",
      name,
      match: { kind: "ambiguous", paths: matches.map((entry) => entry.path) },
    };
  }
  return {
    mode: "byName",
    name,
    match: { kind: "found", path: only.path, resolved: factsOf(found, backend) },
  };
}
