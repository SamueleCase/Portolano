import { type CSSProperties, type KeyboardEvent, useId, useState } from "react";

import { texts } from "../i18n/it";
import { tokens } from "../design/tokens";
import { classNames } from "./classNames";
import styles from "./ColorPicker.module.css";
import { nodeColor } from "./nodeColor";
import { TextField } from "./TextField";
import { useRovingRadio } from "./useRovingRadio";

/** The six colors a node can take with one click (ambiguity 15). */
const swatches = [
  tokens["--color-swatch-1"],
  tokens["--color-swatch-2"],
  tokens["--color-swatch-3"],
  tokens["--color-swatch-4"],
  tokens["--color-swatch-5"],
  tokens["--color-swatch-6"],
];

interface ColorPickerProps {
  /** `#RRGGBB`, or `null` for no color. */
  value: string | null;
  onChange: (value: string | null) => void;
}

type Choice = { kind: "none" } | { kind: "swatch"; color: string } | { kind: "custom" };

function swatchStyle(color: string): CSSProperties & { "--swatch": string } {
  return { "--swatch": color };
}

/**
 * The color of a node: six swatches, "Nessuno" and "Personalizzato…" with a
 * hexadecimal field (ambiguity 15). An imported color outside the six shows
 * as the custom one.
 */
export function ColorPicker({ value, onChange }: ColorPickerProps) {
  const t = texts.components.colorPicker;
  const id = useId();
  const current = nodeColor(value);
  const preset = swatches.find((swatch) => swatch.toLowerCase() === current?.toLowerCase());
  const [editing, setEditing] = useState(Boolean(current && !preset));
  const [draft, setDraft] = useState(current && !preset ? current : "");

  const choices: Choice[] = [
    { kind: "none" },
    ...swatches.map((color): Choice => ({ kind: "swatch", color })),
    { kind: "custom" },
  ];
  const checked = (choice: Choice) =>
    choice.kind === "none"
      ? !current && !editing
      : choice.kind === "swatch"
        ? preset === choice.color && !editing
        : editing || Boolean(current && !preset);

  const choose = (choice: Choice) => {
    setEditing(choice.kind === "custom");
    if (choice.kind === "none") onChange(null);
    if (choice.kind === "swatch") onChange(choice.color);
  };
  const radio = useRovingRadio({
    count: choices.length,
    selected: choices.findIndex(checked),
    onMove: (index) => {
      const next = choices[index];
      if (next) choose(next);
    },
  });
  const commitDraft = (text: string) => {
    setDraft(text);
    const color = nodeColor(text);
    if (color) onChange(color);
  };

  return (
    <div className={styles.picker}>
      <span id={`${id}-label`} className={styles.label}>
        {t.label}
      </span>
      <div role="radiogroup" aria-labelledby={`${id}-label`} className={styles.choices}>
        {choices.map((choice, index) => {
          const on = checked(choice);
          const common = {
            type: "button" as const,
            role: "radio",
            "aria-checked": on,
            tabIndex: radio.tabIndexOf(index),
            ref: radio.register(index),
            onClick: () => choose(choice),
            onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => radio.keyDown(event, index),
          };
          if (choice.kind === "swatch") {
            return (
              <button
                key={choice.color}
                {...common}
                aria-label={t.swatches[index - 1]}
                className={styles.swatch}
                style={swatchStyle(choice.color)}
              />
            );
          }
          const custom = choice.kind === "custom";
          return (
            <button
              key={choice.kind}
              {...common}
              className={classNames(
                styles.option,
                custom && current && !preset && styles.withColor,
              )}
              style={custom && current && !preset ? swatchStyle(current) : undefined}
            >
              {custom ? t.custom : t.none}
            </button>
          );
        })}
      </div>
      {editing && (
        <TextField
          label={t.customField}
          mono
          value={draft}
          placeholder={t.customPlaceholder}
          maxLength={7}
          message={
            draft && !nodeColor(draft) ? { tone: "error", text: t.customInvalid } : undefined
          }
          onChange={(event) => commitDraft(event.target.value)}
        />
      )}
    </div>
  );
}
