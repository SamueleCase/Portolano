import { type KeyboardEvent, useId, useState } from "react";

import { texts } from "../i18n/it";
import { Tag } from "./Tag";
import styles from "./TagInput.module.css";

interface TagInputProps {
  label: string;
  tags: readonly string[];
  onChange: (tags: string[]) => void;
}

/**
 * The tags of a node in the editor (inventario §4.26): Enter or a comma adds
 * the tag written, Backspace on the empty field removes the last one.
 */
export function TagInput({ label, tags, onChange }: TagInputProps) {
  const id = useId();
  const [draft, setDraft] = useState("");

  const add = () => {
    const tag = draft.trim();
    if (tag && !tags.includes(tag)) onChange([...tags, tag]);
    setDraft("");
  };
  const keyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter" || event.key === ",") {
      event.preventDefault();
      add();
    } else if (event.key === "Backspace" && draft === "" && tags.length > 0) {
      onChange(tags.slice(0, -1));
    }
  };

  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
      </label>
      <div className={styles.box}>
        {tags.map((tag) => (
          <Tag
            key={tag}
            name={tag}
            variant="editor"
            onRemove={() => onChange(tags.filter((other) => other !== tag))}
          />
        ))}
        <input
          id={id}
          className={styles.input}
          value={draft}
          placeholder={texts.components.tagInput.placeholder}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={keyDown}
          onBlur={add}
        />
      </div>
    </div>
  );
}
