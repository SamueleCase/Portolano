import styles from "./CommandPreview.module.css";
import { visibleText } from "./visibleText";

interface CommandPreviewProps {
  /** "Comando che verrà lanciato. La password non compare: …". */
  caption: string;
  /** The rendered plan of the backend (`connect_preview`), with `<vault:…>` in place of secrets. */
  command: string;
}

/** The command of a connection, opaque like the terminal (inventario §4.26). */
export function CommandPreview({ caption, command }: CommandPreviewProps) {
  return (
    <figure className={styles.preview}>
      <figcaption className={styles.caption}>{caption}</figcaption>
      <code className={styles.command}>{visibleText(command)}</code>
    </figure>
  );
}
