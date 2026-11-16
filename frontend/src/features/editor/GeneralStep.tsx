import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "../../components/Button";
import { ColorPicker } from "../../components/ColorPicker";
import { FavoriteToggle } from "../../components/FavoriteToggle";
import { GroupHeader } from "../../components/GroupHeader";
import { IconTile } from "../../components/IconTile";
import { TagInput } from "../../components/TagInput";
import { TextArea } from "../../components/TextArea";
import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import { customIconsQuery } from "../../ipc/queries";
import { drawnIcon } from "../dialogs/iconChoice";
import { IconChooser } from "../dialogs/IconChooser";
import { errorMessage, useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import { PositionField } from "./PositionField";
import { PropertiesEditor } from "./PropertiesEditor";

/** Step 1 · Generale (inventario §5.5): what every node has. */
export function GeneralStep() {
  const t = texts.editor.general;
  const { values, set, errorOf } = useEditorFields();
  const [choosingIcon, setChoosingIcon] = useState(false);
  const [iconError, setIconError] = useState<string>();
  const { data: stored = [] } = useQuery(customIconsQuery);

  return (
    <section className={styles.step} aria-label={texts.editor.steps.general}>
      <GroupHeader>{texts.editor.steps.general}</GroupHeader>
      <TextField
        label={t.name}
        value={values.name}
        data-autofocus
        message={errorMessage(errorOf("name"))}
        onChange={(event) => set("name", event.target.value)}
      />
      <PositionField />
      <TextField
        label={t.description}
        value={values.description}
        onChange={(event) => set("description", event.target.value)}
      />
      <TextArea
        label={t.notes}
        value={values.notes}
        onChange={(event) => set("notes", event.target.value)}
      />
      <div className={styles.looks}>
        <div className={styles.group}>
          <span className={styles.fieldLabel}>{t.icon}</span>
          <div className={styles.iconRow}>
            <IconTile
              kind={values.kind}
              icon={drawnIcon(values.icon, stored)}
              color={values.color}
              size="editor"
            />
            <Button onClick={() => setChoosingIcon(true)}>{t.chooseIcon}</Button>
            {values.icon.kind !== "none" && (
              <Button variant="ghost" onClick={() => set("icon", { kind: "none" })}>
                {t.removeIcon}
              </Button>
            )}
          </div>
          {(iconError ?? errorOf("icon")) && (
            <span className={styles.error} role="alert">
              {iconError ?? errorOf("icon")}
            </span>
          )}
        </div>
        <div className={styles.group}>
          <ColorPicker value={values.color} onChange={(color) => set("color", color)} />
          {errorOf("color") && (
            <span className={styles.error} role="alert">
              {errorOf("color")}
            </span>
          )}
        </div>
        <FavoriteToggle
          favorite={values.favorite}
          onChange={(favorite) => set("favorite", favorite)}
        />
      </div>
      <TagInput label={t.tags} tags={values.tags} onChange={(tags) => set("tags", tags)} />
      <PropertiesEditor />
      <IconChooser
        open={choosingIcon}
        value={values.icon}
        onClose={() => setChoosingIcon(false)}
        onError={setIconError}
        onUse={(icon) => {
          setIconError(undefined);
          set("icon", icon);
          setChoosingIcon(false);
        }}
      />
    </section>
  );
}
