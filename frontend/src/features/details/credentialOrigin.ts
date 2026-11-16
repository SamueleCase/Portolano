import type { NodeDetailsView_Serialize, SpecView_Serialize } from "../../bindings";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { pathName, readablePath } from "../tree/treeModel";

/** Where the effective credential of a node comes from, in words (inventario §5.2). */
export interface CredentialOrigin {
  /** The mode chip: "Ereditata dalla cartella Produzione", "Inline", … */
  mode: string;
  /** The text next to it: "credenziale deploy", "usata da 5 nodi", … */
  source: string;
  /**
   * The chain node → folders → credential, when the credential is inherited
   * (`resolve_trace`); untrusted names.
   */
  chain: string[] | null;
  /** Why the credential cannot be resolved, if it cannot (ambiguity 23). */
  problem?: string;
}

/** The chip of a credential reached through `spec`, the last spec of the chain. */
function linkedMode(spec: SpecView_Serialize | undefined): string {
  const origins = texts.details.credential.origins;
  return spec?.mode === "by_name" ? origins.byName : origins.linked;
}

/**
 * The origin of the effective credential of `details`, from its trace; `null`
 * for a node without one (an unsupported node).
 */
export function credentialOrigin(details: NodeDetailsView_Serialize): CredentialOrigin | null {
  const t = texts.details.credential;
  if (details.kind === "credential") {
    const users = details.usage.direct.length + details.usage.inherited.length;
    return { mode: t.origins.own, source: plural(users, t.sources.usedBy), chain: null };
  }
  const trace = details.trace;
  if (trace === null) return null;
  const { steps, end } = trace;
  const last = steps.at(-1);
  const inherited = steps.length > 1 && last !== undefined;
  const inheritedMode = inherited
    ? interpolate(t.origins.inherited, { folder: pathName(last.path) })
    : undefined;
  const chainTo = (target: string) =>
    inherited ? [...steps.map((step) => pathName(step.path)), target] : null;
  const ownMode = (mode: string) =>
    details.kind === "folder" && !inherited ? t.origins.folderDefault : mode;

  switch (end.kind) {
    case "root":
      return { mode: t.origins.root, source: t.sources.root, chain: null };
    case "none":
      return { mode: inheritedMode ?? t.origins.none, source: t.sources.none, chain: null };
    case "prompt":
      return { mode: inheritedMode ?? t.origins.prompt, source: t.sources.prompt, chain: null };
    case "inline":
      return end.node.id === details.id
        ? { mode: ownMode(t.origins.inline), source: t.sources.inlineHere, chain: null }
        : {
            mode: inheritedMode ?? t.origins.inline,
            source: interpolate(t.sources.inlineThere, { path: readablePath(end.node.path) }),
            chain: chainTo(pathName(end.node.path)),
          };
    case "credential":
      return {
        mode: inheritedMode ?? ownMode(linkedMode(last?.spec)),
        source: interpolate(t.sources.credential, { path: readablePath(end.node.path) }),
        chain: chainTo(pathName(end.node.path)),
      };
    case "name_not_found":
      return unresolved(interpolate(t.unresolved.nameNotFound, { name: end.name }));
    case "name_ambiguous":
      return unresolved(
        interpolate(t.unresolved.nameAmbiguous, {
          name: end.name,
          candidates: end.candidates.map((node) => readablePath(node.path)).join(", "),
        }),
      );
    case "dangling":
      return unresolved(t.unresolved.dangling);
    case "not_a_credential":
      return unresolved(
        interpolate(t.unresolved.notACredential, { path: readablePath(end.node.path) }),
      );
  }
}

function unresolved(problem: string): CredentialOrigin {
  return { mode: texts.details.credential.origins.unresolved, source: "", chain: null, problem };
}
