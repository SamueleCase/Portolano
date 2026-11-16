import { Button } from "../components/Button";
import { visibleText } from "../components/visibleText";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { useWorkspace } from "../state/workspace";
import styles from "./DocumentBanner.module.css";
import { useWindowActions } from "./windowActions";

/**
 * The banner of the document file (SPEC-UI §5.4): changed by another program,
 * with "Ricarica"; gone, with "Ricrea vuoto" (an empty document in its place:
 * the backend has no command to write the one in memory there) and "Apri
 * altro…".
 */
export function DocumentBanner() {
  const t = texts.app.banner;
  const banner = useWorkspace((state) => state.banner);
  const setBanner = useWorkspace((state) => state.setBanner);
  const { reload, recreate, openDocument } = useWindowActions();
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
          <Button variant="primary" onClick={reload}>
            {t.reload}
          </Button>
        ) : (
          <>
            <Button onClick={recreate}>{t.recreate}</Button>
            <Button variant="primary" onClick={openDocument}>
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
