import { texts } from "../i18n/it";
import styles from "./FavoriteToggle.module.css";

/** The "★ Preferito" pill of the editor (inventario §4.26), on or off. */
export function FavoriteToggle({
  favorite,
  onChange,
}: {
  favorite: boolean;
  onChange: (favorite: boolean) => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={favorite}
      className={styles.favorite}
      onClick={() => onChange(!favorite)}
    >
      <span className={styles.star} aria-hidden="true">
        {favorite ? "★" : "☆"}
      </span>
      {texts.components.favorite}
    </button>
  );
}
