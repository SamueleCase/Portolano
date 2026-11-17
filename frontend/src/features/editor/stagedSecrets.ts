import type { SlotKey } from "./secretSlots";

/**
 * The secrets typed in the editor and confirmed with "Imposta", until
 * "Salva" (SPEC-UI §1.2, DEC-038.2). A private container of the editor, never
 * React state, never rendered, never in the form, a store or Query: the form
 * holds only the mark "Da salvare". Every "Salva" sent takes all of it out,
 * even if the save then fails; "Annulla", closing and unmounting empty it. A
 * "Salva" stopped before sending (the user does not unlock the vault) keeps
 * it: no value left the editor (DEC-040.4).
 */
export class StagedSecrets {
  private readonly values = new Map<SlotKey, string>();

  stage(key: SlotKey, value: string): void {
    this.values.set(key, value);
  }

  drop(key: SlotKey): void {
    this.values.delete(key);
  }

  /** Every staged value, for the one command of "Salva"; the container is empty after. */
  take(): ReadonlyMap<SlotKey, string> {
    const taken = new Map(this.values);
    this.values.clear();
    return taken;
  }

  clear(): void {
    this.values.clear();
  }

  get size(): number {
    return this.values.size;
  }
}
