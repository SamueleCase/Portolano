import { useQuery } from "@tanstack/react-query";
import { useCallback, useId } from "react";

import { useShortcuts } from "../../app/useWindowCommands";
import { useWindowActions } from "../../app/windowActions";
import type { WindowCommand } from "../../app/windowCommands";
import { Logo } from "../../components/Logo";
import { Spinner } from "../../components/Spinner";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { useBackendEvents } from "../../ipc/events";
import { recentDocumentsQuery } from "../../ipc/queries";
import styles from "./FirstRun.module.css";
import { useDocumentActions } from "./useDocumentActions";

/**
 * The first run (inventario §5.12): no document is open. A new document or an
 * existing one, both chosen with the native dialog (SPEC-UI §4.6), or one of
 * the recent ones. Nothing else is on screen, so the card is not modal and the
 * background stays at full intensity behind it.
 */
export function FirstRun() {
  const t = texts.firstRun;
  const titleId = useId();
  const { createNew, openChosen, openRecent, busy } = useDocumentActions();
  const { openPreferences } = useWindowActions();
  const recent = useQuery(recentDocumentsQuery);
  const recents = recent.data?.filter((document) => document.exists) ?? [];

  // Without a document only the File menu and the preferences make sense.
  const run = useCallback(
    (command: WindowCommand) => {
      if (command.kind === "newDocument") void createNew();
      if (command.kind === "openDocument") void openChosen();
      if (command.kind === "openRecent") void openRecent(command.path);
      if (command.kind === "preferences") openPreferences();
    },
    [createNew, openChosen, openRecent, openPreferences],
  );
  useBackendEvents({ onMenu: run });
  useShortcuts(run);

  return (
    <main className={styles.screen}>
      <section className={styles.card} aria-labelledby={titleId}>
        <h1 id={titleId} className={styles.brand}>
          <Logo size="first-run" />
        </h1>
        <p className={styles.lead}>{t.lead}</p>
        <button
          type="button"
          className={styles.primary}
          aria-busy={busy === "new" || undefined}
          onClick={() => void createNew()}
        >
          <span className={styles.choiceTitle}>
            {t.create.title} {busy === "new" && <Spinner />}
          </span>
          <span className={styles.choiceText}>{t.create.text}</span>
        </button>
        <button
          type="button"
          className={styles.secondary}
          aria-busy={busy === "open" || undefined}
          onClick={() => void openChosen()}
        >
          <span className={styles.choiceTitle}>
            {t.open.title} {busy === "open" && <Spinner />}
          </span>
          <span className={styles.choiceText}>{t.open.text}</span>
        </button>
        {recents.length > 0 && (
          <nav aria-label={t.recent} className={styles.recents}>
            <h2 className={styles.recentTitle}>{t.recent}</h2>
            <ul className={styles.recentList}>
              {recents.map((document) => (
                <li key={document.path}>
                  <button
                    type="button"
                    className={styles.recent}
                    onClick={() => void openRecent(document.path)}
                  >
                    {visibleText(document.path)}
                  </button>
                </li>
              ))}
            </ul>
          </nav>
        )}
      </section>
    </main>
  );
}
