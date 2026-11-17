import type {
  DetailsInput_Deserialize,
  NewNodeInput_Deserialize,
  NodeEditInput_Deserialize,
  SecretProperty,
  StagedSecret,
  StagedSecretsInput_Deserialize,
} from "../../bindings";
import { type EditableField, type EditorValues, sameValue } from "./editorValues";
import { PASSPHRASE, PASSWORD, PRIVATE_KEY, type SlotKey, propertySlot } from "./secretSlots";
import { integerOf, type JsonObject, settingsOf } from "./settingsOf";

/**
 * The commands of "Salva" from the form: `add_node` for a new node,
 * `edit_node` with only what changed for an existing one, the staged
 * secrets inside them (SPEC-UI §3.6). `staged` holds the values taken out of
 * the editor for this one call.
 */

type Staged = ReadonlyMap<SlotKey, string>;

const SETTINGS_FIELDS: readonly EditableField[] = [
  "jumpHosts",
  "agentForwarding",
  "compression",
  "x11",
  "x11Trusted",
  "keepalive",
  "options",
  "resolution",
  "width",
  "height",
  "scale",
  "colorDepth",
  "gatewayHost",
  "gatewayPort",
  "gatewayCredential",
  "multimon",
  "admin",
  "clipboard",
  "wolMac",
  "client",
];

function isConnection(values: EditorValues): boolean {
  return values.kind === "ssh" || values.kind === "rdp";
}

function textProperties(values: EditorValues) {
  return values.properties
    .filter((row) => !row.secret)
    .map((row) => ({ name: row.name.trim(), value: row.text }));
}

function secretProperties(values: EditorValues, staged: Staged): SecretProperty[] {
  return values.properties.flatMap((row) => {
    const value = staged.get(propertySlot(row));
    return row.secret && value !== undefined ? [{ name: row.name.trim(), value }] : [];
  });
}

/** A staged change of the secret in `key`, if any. */
function stagedChange(values: EditorValues, staged: Staged, key: SlotKey): StagedSecret | null {
  const value = staged.get(key);
  if (values.secrets[key] === "set" && value !== undefined) return { action: "set", value };
  if (values.secrets[key] === "clear") return { action: "clear" };
  return null;
}

/** The password of a folder or a connection exists only with an inline credential. */
function passwordChange(values: EditorValues, staged: Staged): StagedSecret | null {
  if (values.kind !== "credential" && values.credential.mode !== "inline") return null;
  return stagedChange(values, staged, PASSWORD);
}

/** The private key in the vault, which another key mode removes. */
function privateKeyChange(
  values: EditorValues,
  staged: Staged,
  storedInVault: boolean,
): StagedSecret | null {
  if (values.keyMode === "vault") return stagedChange(values, staged, PRIVATE_KEY);
  return storedInVault ? { action: "clear" } : null;
}

export function newNodeInput(values: EditorValues, staged: Staged): NewNodeInput_Deserialize {
  const details: DetailsInput_Deserialize = {
    description: values.description,
    notes: values.notes,
    favorite: values.favorite,
    tags: values.tags,
    properties: textProperties(values),
    ...(values.icon.kind === "none" ? {} : { icon: values.icon }),
    ...(values.color === null ? {} : { color: values.color }),
  };
  const set = (change: StagedSecret | null) => (change?.action === "set" ? change.value : null);
  const kind: NewNodeInput_Deserialize["kind"] =
    values.kind === "folder"
      ? { type: "folder", credential: values.credential }
      : values.kind === "credential"
        ? {
            type: "credential",
            username: values.username.trim() || null,
            domain: values.domain.trim() || null,
            keyFile: values.keyMode === "file" ? values.keyFile.trim() : null,
          }
        : {
            type: "connection",
            protocol: values.kind,
            host: values.host.trim(),
            port: integerOf(values.port) ?? null,
            credential: values.credential,
            settings: settingsOf(values),
          };
  return {
    parent: values.parent,
    name: values.name.trim(),
    kind,
    details,
    secrets: {
      password: set(passwordChange(values, staged)),
      passphrase:
        values.kind === "credential" && values.keyMode !== "none"
          ? set(stagedChange(values, staged, PASSPHRASE))
          : null,
      privateKey:
        values.kind === "credential" ? set(privateKeyChange(values, staged, false)) : null,
      properties: secretProperties(values, staged),
    },
  };
}

function detailsChanges(values: EditorValues, base: EditorValues): DetailsInput_Deserialize {
  const changed = (field: EditableField) => !sameValue(values[field], base[field]);
  const kept = new Set(values.properties.map((row) => row.saved));
  return {
    ...(changed("description") ? { description: values.description } : {}),
    ...(changed("notes") ? { notes: values.notes } : {}),
    ...(changed("icon") ? { icon: values.icon } : {}),
    ...(changed("color") ? { color: values.color ?? "" } : {}),
    ...(changed("favorite") ? { favorite: values.favorite } : {}),
    ...(changed("tags") ? { tags: values.tags } : {}),
    ...(sameValue(textProperties(values), textProperties(base))
      ? {}
      : { properties: textProperties(values) }),
    removeSecretProperties: base.properties.flatMap((row) =>
      row.saved !== null && !kept.has(row.saved) ? [row.saved] : [],
    ),
  };
}

function credentialNodeChanges(
  values: EditorValues,
  base: EditorValues,
  staged: Staged,
  stored: (key: SlotKey) => boolean,
): Partial<NodeEditInput_Deserialize> {
  const changed = (field: EditableField) => !sameValue(values[field], base[field]);
  const keyFile = values.keyMode === "file" ? values.keyFile.trim() : "";
  // Without a key its passphrase has no use: a saved one goes too.
  const passphrase =
    values.keyMode === "none"
      ? stored(PASSPHRASE)
        ? ({ action: "clear" } as const)
        : null
      : stagedChange(values, staged, PASSPHRASE);
  return {
    ...(changed("username") ? { username: values.username.trim() } : {}),
    ...(changed("domain") ? { domain: values.domain.trim() } : {}),
    ...(keyFile !== base.keyFile ? { keyFile } : {}),
    secrets: {
      password: stagedChange(values, staged, PASSWORD),
      passphrase,
      privateKey: privateKeyChange(values, staged, base.keyMode === "vault"),
      properties: secretProperties(values, staged),
    },
  };
}

/** What `edit_node` receives: only the fields that differ from the node as loaded. */
export function editInput(
  values: EditorValues,
  base: EditorValues,
  /** The settings stored in the connection, kept where the editor does not write. */
  storedSettings: JsonObject | undefined,
  staged: Staged,
  /** Whether the saved node has a secret in a slot. */
  stored: (key: SlotKey) => boolean,
): NodeEditInput_Deserialize {
  const changed = (field: EditableField) => !sameValue(values[field], base[field]);
  const edit: NodeEditInput_Deserialize = {
    ...(changed("name") ? { name: values.name.trim() } : {}),
    // The move goes in the same save as the rest (DEC-040.2).
    ...(changed("parent") ? { parent: { id: values.parent } } : {}),
    details: detailsChanges(values, base),
  };
  if (values.kind === "credential") {
    return { ...edit, ...credentialNodeChanges(values, base, staged, stored) };
  }
  const secrets: StagedSecretsInput_Deserialize = {
    password: passwordChange(values, staged),
    properties: secretProperties(values, staged),
  };
  const port = integerOf(values.port);
  return {
    ...edit,
    ...(changed("credential") ? { credential: values.credential } : {}),
    ...(isConnection(values) && changed("host") ? { host: values.host.trim() } : {}),
    ...(isConnection(values) && changed("port")
      ? port === undefined
        ? { defaultPort: true }
        : { port }
      : {}),
    ...(isConnection(values) && SETTINGS_FIELDS.some(changed)
      ? { settings: settingsOf(values, storedSettings) }
      : {}),
    secrets,
  };
}
