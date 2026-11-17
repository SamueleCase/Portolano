import type { SecretState } from "../../components/SecretBadge";
import type { SecretSlotInput, SecretView, SecretsBadge } from "../../bindings";
import type { PropertyRow, SecretMark } from "./editorValues";

/**
 * The secrets of the editor by slot: `password`, `passphrase`, `privateKey`,
 * or `prop:<row id>` for a secret property (its name may change before it is
 * saved). A slot names where a secret goes, never what it is.
 */
export type SlotKey = string;

export const PASSWORD: SlotKey = "password";
export const PASSPHRASE: SlotKey = "passphrase";
export const PRIVATE_KEY: SlotKey = "privateKey";

/** The slot of a secret property row. */
export function propertySlot(row: PropertyRow): SlotKey {
  return `prop:${row.id}`;
}

/** The slot of the secret commands (`set_secret`, the dialog) for `key`. */
export function slotInput(key: SlotKey, rows: readonly PropertyRow[]): SecretSlotInput {
  if (key === PASSWORD) return { kind: "password" };
  if (key === PASSPHRASE) return { kind: "passphrase" };
  if (key === PRIVATE_KEY) return { kind: "privateKey" };
  const row = rows.find((candidate) => propertySlot(candidate) === key);
  return { kind: "property", name: row?.name ?? "" };
}

/** What the vault says now of the secret in `key`, if the node has one there. */
function storedState(
  key: SlotKey,
  secrets: readonly SecretView[],
  rows: readonly PropertyRow[],
): SecretsBadge | undefined {
  const row = rows.find((candidate) => propertySlot(candidate) === key);
  const found = secrets.find(({ slot }) => {
    if (typeof slot === "object") return row?.saved !== null && slot.property === row?.saved;
    return slot === (key === PRIVATE_KEY ? "private_key" : key);
  });
  return found?.state;
}

/** The secrets of the node as saved, and the staged changes to them. */
export interface SecretSlots {
  /** The badge of `key`: "Da salvare" once staged, "Vault bloccato" with the vault locked. */
  state: (key: SlotKey) => SecretState;
  /** Whether the saved node has a secret in `key` ("Cancella" then removes it on "Salva"). */
  stored: (key: SlotKey) => boolean;
  /**
   * Whether a value set in `key` replaces one ("Sostituisci"): one staged, or
   * the saved one not cleared, even if the locked vault cannot show it.
   */
  replaces: (key: SlotKey) => boolean;
}

export function secretSlots(
  secrets: readonly SecretView[],
  rows: readonly PropertyRow[],
  marks: Readonly<Record<SlotKey, SecretMark>>,
): SecretSlots {
  return {
    state: (key) => {
      const mark = marks[key];
      if (mark === "set") return "staged";
      if (mark === "clear") return "absent";
      const stored = storedState(key, secrets, rows);
      if (stored === "vaultLocked") return "locked";
      return stored === "present" ? "present" : "absent";
    },
    stored: (key) => storedState(key, secrets, rows) !== undefined,
    replaces: (key) =>
      marks[key] === "set" ||
      (marks[key] !== "clear" && storedState(key, secrets, rows) !== undefined),
  };
}
