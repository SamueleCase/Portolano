import styles from "./Editor.module.css";

interface GroupErrorProps {
  /** The id the controls of the group name in `aria-describedby`. */
  id: string;
  text: string | undefined;
}

/** The error of a group of controls of the editor, which describes them; nothing without one. */
export function GroupError({ id, text }: GroupErrorProps) {
  if (!text) return null;
  return (
    <span id={id} className={styles.error} role="alert">
      {text}
    </span>
  );
}
