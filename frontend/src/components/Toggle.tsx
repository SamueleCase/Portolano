import styles from "./Toggle.module.css";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** The accessible name, when no visible label names the switch. */
  label?: string | undefined;
  /** The id of the visible text that names the switch. */
  labelledBy?: string | undefined;
  describedBy?: string | undefined;
  /** "X11 trusted" can be switched on only with X11 on (inventario §4.5). */
  disabled?: boolean | undefined;
  id?: string | undefined;
}

/** A switch 34×20 (inventario §4.5), a `button` with `role="switch"`. */
export function Toggle({
  checked,
  onChange,
  label,
  labelledBy,
  describedBy,
  disabled = false,
  id,
}: ToggleProps) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      disabled={disabled}
      className={styles.toggle}
      onClick={() => onChange(!checked)}
    >
      <span className={styles.knob} aria-hidden="true" />
    </button>
  );
}
