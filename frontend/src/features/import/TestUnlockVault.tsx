import { commands } from "../../bindings";
import { TEST_MASTER_PASSWORD } from "../../test/fakeBackend";

/**
 * In the tests of the wizard, the unlock dialog of UI-WP5 at its hook point:
 * "Sblocca" opens the fake vault with the test master password (SPEC §1.3).
 */
export function TestUnlockVault({ onDone }: { onDone: (done: boolean) => void }) {
  return (
    <div role="dialog" aria-label="Sblocca il vault">
      <button
        type="button"
        onClick={() =>
          void commands
            .unlockVault(TEST_MASTER_PASSWORD)
            .then((result) => onDone(result.status === "ok"))
        }
      >
        Sblocca
      </button>
      <button type="button" onClick={() => onDone(false)}>
        Annulla
      </button>
    </div>
  );
}
