import type { SecretState } from "./SecretBadge";

/** The modes of a credential (inventario §4.24). */
export type CredentialMode = "inherit" | "none" | "ask" | "inline" | "linked" | "byName";

/** The modes in the order of the selector. */
export const credentialModes: readonly CredentialMode[] = [
  "inherit",
  "none",
  "ask",
  "inline",
  "linked",
  "byName",
];

/** What a credential resolves to: never a secret, only states. */
export interface ResolvedCredential {
  user?: string | undefined;
  domain?: string | undefined;
  password?: SecretState | undefined;
  /** The private key, in words, such as "Nel portachiavi, con passphrase". */
  key?: string | undefined;
}

/** What "Da dove arriverà" shows for each mode (inventario §5.6, ambiguity 23). */
export type CredentialPreviewData =
  | {
      mode: "inherit";
      /** The node being edited. */
      node: string;
      /** The chain node → folders → credential, or `null` if no folder has one. */
      source: { chain: readonly string[]; credential: ResolvedCredential } | null;
    }
  | { mode: "none" }
  | { mode: "ask" }
  | { mode: "inline"; user: string; domain: string; password: SecretState }
  | {
      mode: "linked";
      credential: { name: string; path: string; resolved: ResolvedCredential } | null;
    }
  | {
      mode: "byName";
      name: string;
      match:
        | { kind: "found"; path: string; resolved: ResolvedCredential }
        | { kind: "notFound" }
        | { kind: "ambiguous"; paths: readonly string[] };
    };

/** The callbacks of the parts of the preview that can be edited. */
export interface CredentialPreviewActions {
  onInlineChange?: (field: "user" | "domain", value: string) => void;
  onSetPassword?: () => void;
  onChooseCredential?: () => void;
  onNameChange?: (name: string) => void;
}
