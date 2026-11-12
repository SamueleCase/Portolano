import { type KeyboardEvent, useLayoutEffect, useRef, useState } from "react";

import { classNames } from "./classNames";
import styles from "./SegmentedControl.module.css";

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
  const refs = useRef(new Map<T, HTMLButtonElement>());
  const [indicator, setIndicator] = useState<Indicator>();

  useLayoutEffect(() => {
    const chosen = refs.current.get(value);
    if (chosen) setIndicator({ left: chosen.offsetLeft, width: chosen.offsetWidth });
  }, [value, options]);

  const available = options.filter((option) => !option.unavailable);
  const keyDown = (event: KeyboardEvent) => {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key];
    if (step === undefined || available.length === 0) return;
    event.preventDefault();
    const current = available.findIndex((option) => option.value === value);
    const next = available[(current + step + available.length) % available.length];
    if (next) {
      onChange(next.value);
      refs.current.get(next.value)?.focus();
    }
  };

  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={classNames(styles.group, fullWidth && styles.fullWidth)}
      onKeyDown={keyDown}
    >
      {indicator && (
        <span
          className={styles.indicator}
          aria-hidden="true"
          style={{ transform: `translateX(${indicator.left}px)`, width: indicator.width }}
        />
      )}
      {options.map((option) => {
        const chosen = option.value === value;
        return (
          <button
            key={option.value}
            ref={(element) => {
              if (element) refs.current.set(option.value, element);
              else refs.current.delete(option.value);
            }}
            type="button"
            role="radio"
            aria-checked={chosen}
            aria-disabled={option.unavailable || undefined}
            tabIndex={chosen ? 0 : -1}
            className={classNames(
              styles.option,
              chosen && styles.chosen,
              option.unavailable && styles.unavailable,
            )}
            onClick={() => {
              if (!option.unavailable) onChange(option.value);
            }}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
