import type {
  NodeDetailsView_Serialize,
  SecretSlot,
  SecretSlotInput,
  SecretsBadge,
} from "../../bindings";
import type { SecretState } from "../../components/SecretBadge";
import { texts } from "../../i18n/it";

/** A secret of the node itself, as the detail lists it. */
export interface SecretEntry {
  slot: SecretSlotInput;
  label: string;
  /** `undefined`: not set yet, it can be. */
  state: SecretState | undefined;
}

/** The slot of a secret as the secret commands name it. */
export function slotInput(slot: SecretSlot): SecretSlotInput {
  if (typeof slot === "object") return { kind: "property", name: slot.property };
  return slot === "private_key" ? { kind: "privateKey" } : { kind: slot };
}

function label(slot: SecretSlotInput): string {
  return slot.kind === "property" ? slot.name : texts.details.secrets.slots[slot.kind];
}

const badge: Record<SecretsBadge, SecretState | undefined> = {
  none: undefined,
  present: "present",
  absent: "absent",
  vaultLocked: "locked",
};

/**
 * The secrets of `details` (states only, SPEC-UI §1.1): those set on the node,
 * and the password a credential or an inline login can still receive.
 */
export function secretEntries(details: NodeDetailsView_Serialize): SecretEntry[] {
  const entries = details.secrets.map(({ slot, state }) => {
    const input = slotInput(slot);
    return { slot: input, label: label(input), state: badge[state] };
  });
  const holdsPassword = details.kind === "credential" || details.credentialSpec?.mode === "inline";
  if (holdsPassword && !entries.some((entry) => entry.slot.kind === "password")) {
    entries.unshift({
      slot: { kind: "password" },
      label: label({ kind: "password" }),
      state: undefined,
    });
  }
  return entries;
}
