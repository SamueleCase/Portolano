import { type Segment, SegmentedControl } from "../../components/SegmentedControl";
import styles from "./Preferences.module.css";

interface ChoiceSettingProps<T extends string> {
  label: string;
  options: readonly Segment<T>[];
  value: T;
  onChange: (value: T) => void;
  /** The message of the backend if the last change was refused. */
  error?: string | undefined;
}

/** A labelled segmented control of the preferences, with its error. */
export function ChoiceSetting<T extends string>({
  label,
  options,
  value,
  onChange,
  error,
}: ChoiceSettingProps<T>) {
  return (
    <div className={styles.control}>
      <span className={styles.label} aria-hidden="true">
        {label}
      </span>
      <SegmentedControl label={label} options={options} value={value} onChange={onChange} />
      {error && (
        <span className={styles.error} role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
