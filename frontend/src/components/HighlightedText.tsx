import styles from "./HighlightedText.module.css";
import { visibleText } from "./visibleText";

interface HighlightedTextProps {
  /** Untrusted text of the document. */
  text: string;
  /** The search, marked where it first appears, ignoring case. */
  match?: string | undefined;
}

/** A name with the part found by the search in the accent color (inventario §2.3). */
export function HighlightedText({ text, match }: HighlightedTextProps) {
  const shown = visibleText(text);
  const needle = match ? visibleText(match) : "";
  const start = needle ? shown.toLocaleLowerCase("it").indexOf(needle.toLocaleLowerCase("it")) : -1;
  if (start < 0) {
    return <>{shown}</>;
  }
  const end = start + needle.length;
  return (
    <>
      {shown.slice(0, start)}
      <mark className={styles.match}>{shown.slice(start, end)}</mark>
      {shown.slice(end)}
    </>
  );
}
