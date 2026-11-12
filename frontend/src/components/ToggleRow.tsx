import { useId } from "react";

import { Toggle } from "./Toggle";
import styles from "./ToggleRow.module.css";

interface ToggleRowProps {
  label: string;
  description?: string | undefined;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean | undefined;
}

/** A label, an optional description and the switch on the right (inventario §4.5). */
export function ToggleRow({ label, description, checked, onChange, disabled }: ToggleRowProps) {
  const id = useId();
  return (
    <div className={styles.row}>
      <div className={styles.text}>
        <span id={`${id}-label`} className={styles.label}>
          {label}
        </span>
        {description && (
          <span id={`${id}-description`} className={styles.description}>
            {description}
          </span>
        )}
      </div>
      <Toggle
        checked={checked}
        onChange={onChange}
        labelledBy={`${id}-label`}
        describedBy={description ? `${id}-description` : undefined}
        disabled={disabled}
      />
    </div>
  );
}
