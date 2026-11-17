import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";

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
import { GroupError } from "./GroupError";
import { PositionField } from "./PositionField";
import { PropertiesEditor } from "./PropertiesEditor";

/** Step 1 · Generale (inventario §5.5): what every node has. */
export function GeneralStep() {
  const t = texts.editor.general;
  const { values, set, errorOf } = useEditorFields();
  const [choosingIcon, setChoosingIcon] = useState(false);
  const { data: stored = [] } = useQuery(customIconsQuery);
  const iconLabelId = useId();
  const iconErrorId = useId();
  const colorErrorId = useId();
  const iconError = errorOf("icon");
  const colorError = errorOf("color");

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
        <div
          className={styles.group}
          role="group"
          aria-labelledby={iconLabelId}
          aria-describedby={iconError ? iconErrorId : undefined}
        >
          <span id={iconLabelId} className={styles.fieldLabel}>
            {t.icon}
          </span>
          <div className={styles.iconRow}>
            <IconTile
              kind={values.kind}
              icon={drawnIcon(values.icon, stored)}
              color={values.color}
              size="editor"
            />
            <Button
              aria-describedby={iconError ? iconErrorId : undefined}
              onClick={() => setChoosingIcon(true)}
            >
              {t.chooseIcon}
            </Button>
            {values.icon.kind !== "none" && (
              <Button variant="ghost" onClick={() => set("icon", { kind: "none" })}>
                {t.removeIcon}
              </Button>
            )}
          </div>
          <GroupError id={iconErrorId} text={iconError} />
        </div>
        {/* The picker names itself: the group only carries its error. */}
        <div
          className={styles.group}
          role="group"
          aria-describedby={colorError ? colorErrorId : undefined}
        >
          <ColorPicker value={values.color} onChange={(color) => set("color", color)} />
          <GroupError id={colorErrorId} text={colorError} />
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
        onUse={(icon) => {
          set("icon", icon);
          setChoosingIcon(false);
        }}
      />
    </section>
  );
}
