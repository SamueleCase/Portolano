import type { EditOutcome, SecretSlotInput, SecretView, VaultStatus_Serialize } from "../bindings";
import {
  type FakeBackendState,
  type FakeNode,
  type SlotKey,
  TEST_MASTER_PASSWORD,
  fail,
  failField,
  findNode,
  openDocument,
  isLocked,
  pathOf,
  requireVaultFor,
  slotField,
} from "./fakeBackendModel";
import { slotOf } from "./fakeBackendDetails";

/** The vault and the secrets of one node. Typed secrets are checked and then dropped: only "present" stays. */

export function vaultStatus(state: FakeBackendState): VaultStatus_Serialize {
  return { ...state.vault, locked: isLocked(state) };
}

export function unlockVault(
  state: FakeBackendState,
  masterPassword: string,
): VaultStatus_Serialize {
  if (state.vault.kind !== "file") {
    failField("vault.backend", "il vault configurato non è un file cifrato");
  }
  if (!state.vault.exists) {
    fail("notFound", `il vault su file '${state.vault.path ?? ""}' non esiste ancora: crealo`);
  }
  if (masterPassword !== TEST_MASTER_PASSWORD) {
    fail("vault", "master password errata o vault manomesso");
  }
  state.vault.locked = false;
  return vaultStatus(state);
}

export function lockVault(state: FakeBackendState): VaultStatus_Serialize {
  if (state.vault.kind === "file") state.vault.locked = true;
  return vaultStatus(state);
}

export function createFileVault(
  state: FakeBackendState,
  masterPassword: string,
): VaultStatus_Serialize {
  if (state.vault.kind !== "file") {
    failField("vault.backend", "il vault configurato non è un file cifrato");
  }
  if (state.vault.exists) fail("invalid", `il file '${state.vault.path ?? ""}' esiste già`);
  if ([...masterPassword].length < 12) {
    fail("vault", "la master password di un nuovo vault deve avere almeno 12 caratteri");
  }
  state.vault.exists = true;
  state.vault.locked = false;
  return vaultStatus(state);
}

function keyOf(slot: SecretSlotInput): SlotKey {
  switch (slot.kind) {
    case "password":
    case "passphrase":
      return slot.kind;
    case "privateKey":
      return "privateKey";
    case "property":
      return `property:${slot.name}`;
  }
}

/** Refuses a slot the node cannot hold, as `check_secret_slot`. */
function checkSlot(node: FakeNode, key: SlotKey): void {
  if (key.startsWith("property:")) {
    const name = key.slice("property:".length);
    if (node.properties.some((property) => property.name === name && property.text !== null)) {
      fail("invalid", `la proprietà '${name}' di '${node.name}' è testuale, non segreta`);
    }
    return;
  }
  const applicable =
    node.kind === "credential" ||
    (key === "password" &&
      (node.kind === "folder" || node.kind === "connection") &&
      node.credential.mode === "inline");
  if (!applicable) {
    failField(slotField(key), `il segreto non è applicabile a '${node.name}'`);
  }
}

export function secretStatus(state: FakeBackendState, id: string): SecretView[] {
  const { node } = findNode(openDocument(state), id);
  // As `secret_status`: the vault is asked only for the secrets the node has.
  requireVaultFor(state, Object.keys(node.secrets).length > 0);
  return Object.entries(node.secrets).map(([key, present]) => ({
    slot: slotOf(key),
    state: present ? "present" : "absent",
  }));
}

export function setSecret(
  state: FakeBackendState,
  id: string,
  slot: SecretSlotInput,
): SecretView[] {
  const { node } = findNode(openDocument(state), id);
  const key = keyOf(slot);
  checkSlot(node, key);
  requireVaultFor(state, true);
  if (
    key.startsWith("property:") &&
    !node.properties.some((property) => `property:${property.name}` === key)
  ) {
    node.properties.push({ name: key.slice("property:".length), text: null });
  }
  node.secrets[key] = true;
  return secretStatus(state, id);
}

export function clearSecret(
  state: FakeBackendState,
  id: string,
  slot: SecretSlotInput,
): EditOutcome {
  const doc = openDocument(state);
  const { node } = findNode(doc, id);
  const key = keyOf(slot);
  checkSlot(node, key);
  if (node.secrets[key] === undefined) {
    failField(slotField(key), `il segreto di '${pathOf(doc, id)}' non è impostato`);
  }
  requireVaultFor(state, true);
  delete node.secrets[key];
  if (key.startsWith("property:")) {
    node.properties = node.properties.filter((property) => `property:${property.name}` !== key);
  }
  return { path: pathOf(doc, id), secrets_written: 0, secrets_deleted: 1, cleanup_failures: [] };
}
