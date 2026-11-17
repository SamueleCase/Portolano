import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { ImportResult, ReportItem } from "../../bindings";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useSaveImportReport } from "../../ipc/mutations";
import { treeQuery } from "../../ipc/queries";
import { importedNode } from "./importedNode";

/**
 * The actions of the report: a row leads to its node through `source_id`
 * (under the imported folder first, then in the whole tree), "Esporta JSON"
 * saves the report where the native dialog says.
 */
export function useReportActions(result: ImportResult | null, onGoToNode: (id: string) => void) {
  const queryClient = useQueryClient();
  const save = useSaveImportReport();
  const [notice, setNotice] = useState<string | null>(null);

  const goToItem = async (item: ReportItem) => {
    if (result === null) return;
    setNotice(null);
    try {
      const tree = await queryClient.fetchQuery(treeQuery);
      const id = importedNode(tree, result.root, item.source_id);
      if (id === null) setNotice(texts.importWizard.report.nodeMissing);
      else onGoToNode(id);
    } catch (failure) {
      setNotice(presentFailure(failure).message);
    }
  };

  const exportJson = async () => {
    setNotice(null);
    try {
      const chosen = await save.mutateAsync();
      if (chosen)
        setNotice(interpolate(texts.importWizard.report.exported, { file: chosen.fileName }));
    } catch (failure) {
      setNotice(presentFailure(failure).message);
    }
  };

  return { notice, goToItem, exportJson, exporting: save.isPending };
}
