import { Button } from "../components/Button";
import { visibleText } from "../components/visibleText";
import { useDocumentActions } from "../features/first-run/useDocumentActions";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { useWorkspace } from "../state/workspace";
import styles from "./DocumentBanner.module.css";

/**
 * The banner of the document file (SPEC-UI §5.4): changed by another program,
 * with "Ricarica"; gone, with "Ricrea" (the document open in memory written
 * back where it was, `recreate_document`) and "Apri altro…".
 */
export function DocumentBanner() {
  const t = texts.app.banner;
  const banner = useWorkspace((state) => state.banner);
  const setBanner = useWorkspace((state) => state.setBanner);
  const { reload, recreate, openChosen, busy } = useDocumentActions();
  if (banner === null) return null;
  const file = visibleText(banner.fileName);
  return (
    <div role="alert" className={styles.banner}>
      <span className={styles.marker} aria-hidden="true" />
      <span className={styles.text}>
        {interpolate(banner.kind === "modified" ? t.modified : t.missing, { file })}
      </span>
      <span className={styles.actions}>
        {banner.kind === "modified" ? (
          <Button variant="primary" loading={busy === "reload"} onClick={() => void reload()}>
            {t.reload}
          </Button>
        ) : (
          <>
            <Button
              variant="primary"
              loading={busy === "recreate"}
              disabled={busy === "open"}
              onClick={() => void recreate()}
            >
              {t.recreate}
            </Button>
            <Button
              loading={busy === "open"}
              disabled={busy === "recreate"}
              onClick={() => void openChosen()}
            >
              {t.openOther}
            </Button>
          </>
        )}
        <button
          type="button"
          className={styles.close}
          aria-label={t.close}
          onClick={() => setBanner(null)}
        >
          ×
        </button>
      </span>
    </div>
  );
}
