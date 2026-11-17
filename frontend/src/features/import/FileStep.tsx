import { Button } from "../../components/Button";
import { Card } from "../../components/Card";
import { visibleText } from "../../components/visibleText";
import { plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./ImportWizard.module.css";

/** What the wizard knows of the chosen file. */
export type FileReading =
  | { state: "reading" }
  | { state: "readable"; objects: number }
  | { state: "unreadable"; message: string };

interface FileStepProps {
  /** The name of the chosen file, none before the native dialog. */
  fileName: string | null;
  reading: FileReading | null;
  /** Opens the native dialog (SPEC-UI §4.6: the path stays in Rust). */
  onChoose: () => void;
  choosing: boolean;
}

/**
 * Step 1 of the wizard (inventario §5.13, §5.17): the native dialog chooses
 * the file, which is read at once, so that an unreadable file shows here.
 * No drop area: the frontend never gets the path of a dropped file.
 */
export function FileStep({ fileName, reading, onChoose, choosing }: FileStepProps) {
  const { file } = texts.importWizard;
  return (
    <div className={styles.step}>
      <Card large className={styles.chooser}>
        <span className={styles.chooserTitle}>{file.title}</span>
        <span className={styles.muted}>{file.hint}</span>
        <Button variant="primary" onClick={onChoose} loading={choosing}>
          {fileName === null ? file.choose : file.chooseAnother}
        </Button>
      </Card>
      {fileName !== null && reading !== null && (
        <div className={styles.fileRow}>
          <span className={styles.fileName}>{visibleText(fileName)}</span>
          <span className={styles.fileFacts} role="status">
            {reading.state === "reading" && file.reading}
            {reading.state === "readable" && (
              <>
                <span className={styles.muted}>{plural(reading.objects, file.read)}</span>
                <span className={styles.ok}>{file.readable}</span>
              </>
            )}
            {reading.state === "unreadable" && (
              <span className={styles.error}>{file.unreadable}</span>
            )}
          </span>
        </div>
      )}
      {reading?.state === "unreadable" && (
        <p role="alert" className={styles.error}>
          {visibleText(reading.message)}
        </p>
      )}
    </div>
  );
}
