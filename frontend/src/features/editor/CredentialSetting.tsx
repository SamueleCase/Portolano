import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";

import { CredentialField } from "../../components/CredentialField";
import type { CredentialMode } from "../../components/credentialPreviewData";
import type { SecretState } from "../../components/SecretBadge";
import type { CredentialChoice_Deserialize, SecretsBadge } from "../../bindings";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { CredentialChooser } from "../dialogs/CredentialChooser";
import { modeOfChoice } from "../dialogs/credentialPreview";
import { indexedNodes } from "../dialogs/treeIndex";
import { useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import type { GatewayCredential } from "./editorValues";
import { GroupError } from "./GroupError";
import { PASSWORD } from "./secretSlots";

interface CredentialSettingProps {
  /** "Credenziale", "Credenziale predefinita", "Credenziale del gateway". */
  label: string;
  value: CredentialChoice_Deserialize | GatewayCredential;
  onChange: (choice: CredentialChoice_Deserialize) => void;
  modes?: readonly CredentialMode[];
  /** Whether an inline credential here holds the password of the node (not the gateway's). */
  ownPassword: boolean;
  error?: string | undefined;
}

function badgeOf(state: SecretsBadge): SecretState | undefined {
  if (state === "none") return undefined;
  return state === "vaultLocked" ? "locked" : state;
}

function login(username: string | null | undefined, domain: string | null | undefined) {
  if (!username) return undefined;
  return domain ? `${domain}\\${username}` : username;
}

/**
 * A credential of the editor: the field with its mode, origin and password
 * (inventario §4.23), and the selector that changes it.
 */
export function CredentialSetting({
  label,
  value,
  onChange,
  modes,
  ownPassword,
  error,
}: CredentialSettingProps) {
  const t = texts.dialogs.credential.origin;
  const { values, secrets, askSecret, backend } = useEditorFields();
  const [choosing, setChoosing] = useState(false);
  const errorId = useId();
  const { data: tree } = useQuery(treeQuery);
  const entries = tree ? indexedNodes(tree) : [];

  const mode =
    value.mode === "kept"
      ? value.stored === "inline"
        ? "inline"
        : "ask"
      : modeOfChoice[value.mode];
  const linked = value.mode === "byId" ? entries.find((entry) => entry.node.id === value.id) : null;
  const origin = (() => {
    switch (value.mode) {
      case "inherit":
      case "none":
      case "prompt":
        return t[value.mode];
      case "byId":
        return linked?.path ?? t.missing;
      case "byName":
        return interpolate(t.byName, { name: value.name });
      case "inline":
      case "kept":
        return undefined;
    }
  })();
  const user =
    value.mode === "inline" || value.mode === "kept"
      ? login(value.username, value.mode === "inline" ? value.domain : null)
      : undefined;
  const password =
    value.mode === "inline" && ownPassword
      ? secrets.state(PASSWORD)
      : linked
        ? badgeOf(linked.node.secrets)
        : undefined;

  return (
    <div className={styles.group}>
      <CredentialField
        label={label}
        mode={texts.components.credentialModes[mode].title}
        origin={origin}
        user={user}
        password={password}
        backend={backend}
        onChange={() => setChoosing(true)}
        describedBy={error ? errorId : undefined}
      />
      <GroupError id={errorId} text={error} />
      <CredentialChooser
        open={choosing}
        nodeName={values.name || texts.editor.untitled}
        parent={values.parent}
        value={value.mode === "kept" ? { mode: "inherit" } : value}
        modes={modes}
        passwordState={secrets.state(PASSWORD)}
        onSetPassword={() => askSecret(PASSWORD)}
        onClose={() => setChoosing(false)}
        onDone={(choice) => {
          onChange(choice);
          setChoosing(false);
        }}
      />
    </div>
  );
}
