import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import {
  type CredentialMode,
  credentialModes,
  type CredentialPreviewData,
} from "../../components/credentialPreviewData";
import { CredentialSelector } from "../../components/CredentialSelector";
import type { SecretState } from "../../components/SecretBadge";
import type { CredentialChoice_Deserialize } from "../../bindings";
import { nodeDetailsQuery, treeQuery } from "../../ipc/queries";
import { byNamePreview, inheritPreview, linkedPreview, modeOfChoice } from "./credentialPreview";
import { TreeNodePicker } from "./TreeNodePicker";
import { credentialsNamed, indexTree } from "./treeIndex";
import { useVaultState } from "./vault";

interface CredentialChooserProps {
  open: boolean;
  /** The name of the node being edited, for "Eredita". */
  nodeName: string;
  /** Its folder (`null` is the root): what "Eredita" reaches. */
  parent: string | null;
  /** The credential chosen now. */
  value: CredentialChoice_Deserialize;
  /** The modes offered, for instance without "Inline" for an RDP gateway. */
  modes?: readonly CredentialMode[] | undefined;
  /** The inline password of the node, as the editor holds it ("Da salvare" once staged). */
  passwordState: SecretState;
  /** "Imposta…" of the inline password: the editor asks it and stages it. */
  onSetPassword: () => void;
  /** "Fatto" with a complete choice. */
  onDone: (choice: CredentialChoice_Deserialize) => void;
  /** Esc, the veil, or "Fatto" with nothing changed or a choice incomplete: nothing changes. */
  onClose: () => void;
}

/** The parts of every mode, kept while the user goes from one mode to another. */
interface Draft {
  mode: CredentialMode;
  user: string;
  domain: string;
  id: string | null;
  name: string;
}

function draftOf(value: CredentialChoice_Deserialize): Draft {
  return {
    mode: modeOfChoice[value.mode],
    user: value.mode === "inline" ? (value.username ?? "") : "",
    domain: value.mode === "inline" ? (value.domain ?? "") : "",
    id: value.mode === "byId" ? value.id : null,
    name: value.mode === "byName" ? value.name : "",
  };
}

/** The choice of `draft`, or `null` while it lacks its credential or its name. */
function choiceOf(draft: Draft): CredentialChoice_Deserialize | null {
  switch (draft.mode) {
    case "inherit":
    case "none":
      return { mode: draft.mode };
    case "ask":
      return { mode: "prompt" };
    case "inline":
      return { mode: "inline", username: draft.user || null, domain: draft.domain || null };
    case "linked":
      return draft.id === null ? null : { mode: "byId", id: draft.id };
    case "byName":
      return draft.name.trim() === "" ? null : { mode: "byName", name: draft.name };
  }
}

/**
 * The credential selector over the open document (inventario §5.6): the six
 * modes, with "Da dove arriverà" from the data of the backend, including the
 * states "nessuna credenziale ereditata", "nome non trovato" and "nome
 * ambiguo" (ambiguity 23).
 */
export function CredentialChooser(props: CredentialChooserProps) {
  // Mounted only while open: every opening starts from the current choice.
  return props.open ? <OpenCredentialChooser {...props} /> : null;
}

function OpenCredentialChooser({
  nodeName,
  parent,
  value,
  modes = credentialModes,
  passwordState,
  onSetPassword,
  onDone,
  onClose,
}: CredentialChooserProps) {
  const { backend } = useVaultState();
  const [draft, setDraft] = useState(() => draftOf(value));
  const [picking, setPicking] = useState(false);
  // Whether the user changed anything: "Fatto" on an untouched selector
  // keeps the credential as it is, also a spec it cannot show (DEC-038.4).
  const [touched, setTouched] = useState(false);
  const change = (part: Partial<Draft>) => {
    setTouched(true);
    setDraft((current) => ({ ...current, ...part }));
  };

  const { data: tree } = useQuery(treeQuery);
  const entries = tree ? indexTree(tree) : [];
  const [onlyMatch] = draft.mode === "byName" ? credentialsNamed(entries, draft.name) : [];
  const parentDetails = useQuery({
    ...nodeDetailsQuery(parent ?? ""),
    enabled: draft.mode === "inherit" && parent !== null,
  });
  const linked = useQuery({
    ...nodeDetailsQuery(draft.id ?? ""),
    enabled: draft.mode === "linked" && draft.id !== null,
  });
  const named = useQuery({
    ...nodeDetailsQuery(onlyMatch?.node.id ?? ""),
    enabled: onlyMatch !== undefined,
  });

  const preview = ((): CredentialPreviewData => {
    switch (draft.mode) {
      case "inherit":
        return inheritPreview(nodeName, parent === null ? null : parentDetails.data, backend);
      case "none":
        return { mode: "none" };
      case "ask":
        return { mode: "ask" };
      case "inline":
        return { mode: "inline", user: draft.user, domain: draft.domain, password: passwordState };
      case "linked":
        return linkedPreview(draft.id === null ? undefined : linked.data, backend);
      case "byName":
        return byNamePreview(draft.name, entries, named.data, backend);
    }
  })();

  return (
    <>
      <CredentialSelector
        open
        onClose={onClose}
        onDone={() => {
          const choice = touched ? choiceOf(draft) : null;
          if (choice) onDone(choice);
          else onClose();
        }}
        mode={draft.mode}
        onModeChange={(mode) => {
          change({ mode });
          // "Da una credenziale" without one yet: the picker comes at once.
          if (mode === "linked" && draft.id === null) setPicking(true);
        }}
        preview={preview}
        backend={backend}
        modes={modes}
        actions={{
          onInlineChange: (field, text) =>
            change(field === "user" ? { user: text } : { domain: text }),
          onSetPassword,
          onChooseCredential: () => setPicking(true),
          onNameChange: (name) => change({ name }),
        }}
      />
      <TreeNodePicker
        open={picking}
        purpose="credential"
        onClose={() => setPicking(false)}
        onChoose={(id) => {
          change({ id });
          setPicking(false);
        }}
      />
    </>
  );
}
