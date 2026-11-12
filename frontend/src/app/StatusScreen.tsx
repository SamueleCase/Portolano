import { useQuery } from "@tanstack/react-query";

import { texts } from "../i18n/it";
import { appStatusQuery } from "../ipc/queries";
import { IpcFailure } from "../ipc/result";
import styles from "./StatusScreen.module.css";

/**
 * The test screen of UI-WP0: what the backend knows at startup. The first-run
 * screen of UI-WP4 replaces it.
 */
export function StatusScreen() {
  const status = useQuery(appStatusQuery);

  return (
    <main className={styles["screen"]}>
      <h1 className={styles["title"]}>{texts.status.title}</h1>
      {status.isPending && <p className={styles["note"]}>{texts.status.loading}</p>}
      {status.isError && (
        <p role="alert" className={styles["error"]}>
          {texts.status.failed}:{" "}
          {status.error instanceof IpcFailure ? status.error.ipc.message : status.error.message}
        </p>
      )}
      {status.isSuccess && (
        <dl className={styles["facts"]}>
          <dt>{texts.status.defaultDocument}</dt>
          <dd>
            <span className={styles["path"]}>{status.data.defaultDocument.path}</span>{" "}
            <span className={styles["note"]}>
              {status.data.defaultDocument.exists
                ? texts.status.documentPresent
                : texts.status.documentAbsent}
            </span>
          </dd>
          <dt>{texts.status.vault}</dt>
          <dd>{texts.status.vaultKinds[status.data.vault]}</dd>
        </dl>
      )}
    </main>
  );
}
