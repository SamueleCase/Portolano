import { type KeyboardEvent, useId, useState } from "react";

import { classNames } from "./classNames";
import fieldStyles from "./field.module.css";
import styles from "./Select.module.css";

/** An option of a select. */
export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

interface SelectProps<T extends string> {
  label: string;
  options: readonly SelectOption<T>[];
  value: T;
  onChange: (value: T) => void;
  mono?: boolean;
}

/**
 * A select drawn as in the DS (inventario §4.4): a field with "▾" that opens a
 * list. It follows the "select-only combobox" pattern of WAI-ARIA: the focus
 * stays on the field and the arrows move the active option.
 */
export function Select<T extends string>({
  label,
  options,
  value,
  onChange,
  mono = false,
}: SelectProps<T>) {
  const id = useId();
  const [open, setOpen] = useState(false);
  const selectedIndex = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  );
  const [active, setActive] = useState(selectedIndex);
  const selected = options[selectedIndex];

  const show = () => {
    setActive(selectedIndex);
    setOpen(true);
  };
  const choose = (index: number) => {
    const option = options[index];
    if (option) onChange(option.value);
    setOpen(false);
  };
  const move = (index: number) => {
    setActive(Math.min(options.length - 1, Math.max(0, index)));
  };

  const keyDown = (event: KeyboardEvent) => {
    const keys: Record<string, () => void> = open
      ? {
          ArrowDown: () => move(active + 1),
          ArrowUp: () => move(active - 1),
          Home: () => move(0),
          End: () => move(options.length - 1),
          Enter: () => choose(active),
          " ": () => choose(active),
          Escape: () => setOpen(false),
          Tab: () => setOpen(false),
        }
      : { ArrowDown: show, ArrowUp: show, Enter: show, " ": show };
    const action = keys[event.key];
    if (!action) return;
    if (event.key !== "Tab") event.preventDefault();
    action();
  };

  return (
    <div className={styles.select}>
      <span id={`${id}-label`} className={styles.label}>
        {label}
      </span>
      <div
        role="combobox"
        tabIndex={0}
        aria-labelledby={`${id}-label`}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={`${id}-list`}
        aria-activedescendant={open ? `${id}-option-${active}` : undefined}
        className={classNames(fieldStyles.control, styles.trigger, mono && fieldStyles.mono)}
        onClick={() => (open ? setOpen(false) : show())}
        onKeyDown={keyDown}
        onBlur={() => setOpen(false)}
      >
        <span>{selected?.label}</span>
        <span className={styles.chevron} aria-hidden="true">
          ▾
        </span>
      </div>
      {open && (
        <ul
          id={`${id}-list`}
          role="listbox"
          aria-labelledby={`${id}-label`}
          className={styles.list}
        >
          {options.map((option, index) => (
            <li
              key={option.value}
              id={`${id}-option-${index}`}
              role="option"
              aria-selected={option.value === value}
              className={classNames(
                styles.option,
                index === active && styles.active,
                option.value === value && styles.chosen,
                mono && fieldStyles.mono,
              )}
              // Keeps the focus on the field, so that its blur does not close the list first.
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => choose(index)}
            >
              {option.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
