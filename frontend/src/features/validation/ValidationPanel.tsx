import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { Button } from "../../components/Button";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { treeQuery, validationQuery } from "../../ipc/queries";
import { readablePaths } from "./treePaths";
import styles from "./Validation.module.css";
import { ValidationReport } from "./ValidationReport";

interface ValidationPanelProps {
  /** "Vai al nodo →" of a row: the host selects the node and shows its detail. */
  onGoToNode: (id: string) => void;
}

/**
 * The content of the validation tab (SPEC-UI §5.4 "Validazione"): the
 * problems of `validate` with the paths of the tree. "Ricontrolla" asks the
 * backend again; with the vault locked the backend sends a single note
 * instead of a missing secret for each password.
 */
export function ValidationPanel({ onGoToNode }: ValidationPanelProps) {
  const validation = useQuery(validationQuery);
  const tree = useQuery(treeQuery);
  const paths = useMemo(
    () => (tree.data ? readablePaths(tree.data) : new Map<string, string>()),
    [tree.data],
  );

  return (
    <div className={styles.panel}>
      {validation.isError ? (
        <>
          <p role="alert" className={styles.failure}>
            {presentFailure(validation.error).message}
          </p>
          <Button
            className={styles.retry}
            onClick={() => void validation.refetch()}
            loading={validation.isFetching}
          >
            {texts.validation.recheck}
          </Button>
        </>
      ) : validation.data === undefined ? (
        <p role="status" className={styles.none}>
          {texts.validation.checking}
        </p>
      ) : (
        <ValidationReport
          issues={validation.data}
          pathOf={(id) => paths.get(id) ?? texts.validation.unknownNode}
          onGoToNode={onGoToNode}
          onRecheck={() => void validation.refetch()}
          checking={validation.isFetching}
        />
      )}
    </div>
  );
}
