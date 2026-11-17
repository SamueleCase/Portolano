import { useCallback, useRef } from "react";

import type { ImportOptionsInput_Deserialize, ImportPreviewView_Serialize } from "../../bindings";
import { presentFailure } from "../../ipc/errors";
import { useImportPreview } from "../../ipc/mutations";
import type { ImportSettings } from "./OptionsStep";

/** The options of the backend for `settings`: "Chiedi le password" is the wizard's alone. */
export function importOptions(settings: ImportSettings): ImportOptionsInput_Deserialize {
  return {
    into: settings.into,
    onConflict: settings.onConflict,
    includeTrash: settings.includeTrash,
  };
}

/** The answer of a dry run: what the import would do, or why it cannot. */
export type PreviewAnswer =
  { status: "ok"; preview: ImportPreviewView_Serialize } | { status: "error"; message: string };

/**
 * The dry runs of the import (`import_preview`). Only the answer to the
 * latest call counts: a slow answer to older settings resolves to `null`,
 * so that it never replaces a newer one.
 */
export function useWizardPreview() {
  const { mutateAsync, isPending } = useImportPreview();
  const latest = useRef(0);

  const run = useCallback(
    async (options: ImportOptionsInput_Deserialize): Promise<PreviewAnswer | null> => {
      latest.current += 1;
      const ticket = latest.current;
      let answer: PreviewAnswer;
      try {
        answer = { status: "ok", preview: await mutateAsync(options) };
      } catch (failure) {
        answer = { status: "error", message: presentFailure(failure).message };
      }
      return ticket === latest.current ? answer : null;
    },
    [mutateAsync],
  );

  return { run, previewing: isPending };
}
