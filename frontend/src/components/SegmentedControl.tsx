import { useLayoutEffect, useRef, useState } from "react";

import { classNames } from "./classNames";
import styles from "./SegmentedControl.module.css";
import { useRovingRadio } from "./useRovingRadio";

/** An option of a segmented control. */
export interface Segment<T extends string> {
  value: T;
  label: string;
  /** Shown but not choosable, such as "Chiaro · in arrivo" (inventario §4.6). */
  unavailable?: boolean;
}

interface SegmentedControlProps<T extends string> {
  /** The accessible name of the group. */
  label: string;
  options: readonly Segment<T>[];
  value: T;
  onChange: (value: T) => void;
  /** Spreads the options over the whole width. */
  fullWidth?: boolean;
}

interface Indicator {
  left: number;
  width: number;
}

/**
 * A segmented control (inventario §4.6) with the semantics of a radio group:
 * one tab stop, the arrows choose the next option. The green fill slides to
 * the chosen option.
 */
export function SegmentedControl<T extends string>({
  label,
  options,
  value,
  onChange,
  fullWidth = false,
}: SegmentedControlProps<T>) {
  const group = useRef<HTMLDivElement>(null);
  const [indicator, setIndicator] = useState<Indicator>();
  const radio = useRovingRadio({
    count: options.length,
    selected: options.findIndex((option) => option.value === value),
    available: (index) => !options[index]?.unavailable,
    onMove: (index) => {
      const option = options[index];
      if (option) onChange(option.value);
    },
  });

  useLayoutEffect(() => {
    const chosen = group.current?.querySelector<HTMLElement>("[aria-checked='true']");
    if (chosen) setIndicator({ left: chosen.offsetLeft, width: chosen.offsetWidth });
  }, [value, options]);

  return (
    <div
      role="radiogroup"
      aria-label={label}
      ref={group}
      className={classNames(styles.group, fullWidth && styles.fullWidth)}
    >
      {indicator && (
        <span
          className={styles.indicator}
          aria-hidden="true"
          style={{ transform: `translateX(${indicator.left}px)`, width: indicator.width }}
        />
      )}
      {options.map((option, index) => {
        const chosen = option.value === value;
        return (
          <button
            key={option.value}
            ref={radio.register(index)}
            type="button"
            role="radio"
            aria-checked={chosen}
            aria-disabled={option.unavailable || undefined}
            tabIndex={radio.tabIndexOf(index)}
            className={classNames(
              styles.option,
              chosen && styles.chosen,
              option.unavailable && styles.unavailable,
            )}
            onClick={() => {
              if (!option.unavailable) onChange(option.value);
            }}
            onKeyDown={(event) => radio.keyDown(event, index)}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
