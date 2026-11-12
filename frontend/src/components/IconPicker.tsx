import { useState } from "react";

import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { Button } from "./Button";
import { Dialog } from "./Dialog";
import { builtinIconNames } from "./icons/builtinIconNames";
import { BuiltinIcon } from "./icons/BuiltinIcon";
import { customIconUrl } from "./icons/customIcon";
import type { NodeIconChoice } from "./icons/NodeIcon";
import { IconGrid, type IconGridTile } from "./IconGrid";
import styles from "./IconPicker.module.css";
import { SegmentedControl } from "./SegmentedControl";
import { TextField } from "./TextField";
import { visibleText } from "./visibleText";

/** An icon of the document's icon library (`IconStore::list()`). */
export interface CustomIconOption {
  fileName: string;
  /** The name it came with, untrusted. */
  label: string;
}

type Source = "builtin" | "custom" | "image";

interface IconPickerProps {
  open: boolean;
  onClose: () => void;
  /** The icon chosen now, if any. */
  value: NodeIconChoice | undefined;
  onUse: (choice: NodeIconChoice) => void;
  customIcons: readonly CustomIconOption[];
  /** "Immagine": Rust opens the native dialog and adds the file (SPEC-UI §4.6). */
  onChooseImage: () => void;
}

function matches(text: string, search: string): boolean {
  return text.toLocaleLowerCase("it").includes(search.trim().toLocaleLowerCase("it"));
}

/**
 * The icon picker (inventario §5.7): the builtin Lucide set, the icons
 * imported with the document (only as `<img>`), or a new image from disk.
 */
export function IconPicker({
  open,
  onClose,
  value,
  onUse,
  customIcons,
  onChooseImage,
}: IconPickerProps) {
  const t = texts.components.iconPicker;
  const [source, setSource] = useState<Source>(value?.source === "custom" ? "custom" : "builtin");
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<NodeIconChoice | undefined>(value);

  const builtinTiles: IconGridTile[] = builtinIconNames
    .filter((name) => matches(texts.icons[name], search) || matches(name, search))
    .map((name) => ({
      id: name,
      label: texts.icons[name],
      icon: <BuiltinIcon name={name} size="tile" />,
      choice: { source: "builtin", name },
    }));
  const customTiles: IconGridTile[] = customIcons.flatMap((icon) => {
    const url = customIconUrl(icon.fileName);
    const label = interpolate(t.customIcon, { name: visibleText(icon.label) });
    if (!url || !matches(icon.label, search)) return [];
    return [
      {
        id: icon.fileName,
        label,
        icon: <img src={url} alt="" className={styles.custom} draggable={false} />,
        choice: { source: "custom", fileName: icon.fileName },
      },
    ];
  });
  const tiles = source === "builtin" ? builtinTiles : customTiles;
  const selectedId =
    selected?.source === "builtin"
      ? selected.name
      : selected?.source === "custom"
        ? selected.fileName
        : undefined;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={t.title}
      width="small"
      footer={
        <>
          <Button onClick={onClose}>{t.cancel}</Button>
          <Button
            variant="primary"
            disabled={!selected}
            onClick={() => {
              if (selected) onUse(selected);
            }}
          >
            {t.use}
          </Button>
        </>
      }
    >
      <SegmentedControl
        label={t.title}
        fullWidth
        value={source}
        onChange={setSource}
        options={[
          { value: "builtin", label: t.sources.builtin },
          { value: "custom", label: t.sources.custom },
          { value: "image", label: t.sources.image },
        ]}
      />
      {source === "image" ? (
        <div className={styles.imageSource}>
          <Button onClick={onChooseImage}>{t.chooseImage}</Button>
          <span className={styles.hint}>{t.imageHint}</span>
        </div>
      ) : (
        <>
          <TextField
            label={t.search}
            variant="search"
            placeholder={t.search}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          {tiles.length > 0 ? (
            <IconGrid
              label={t.title}
              tiles={tiles}
              selectedId={selectedId}
              onSelect={(tile) => setSelected(tile.choice)}
            />
          ) : (
            <p className={styles.empty}>
              {source === "custom" && customIcons.length === 0 ? t.noCustom : t.noMatch}
            </p>
          )}
        </>
      )}
    </Dialog>
  );
}
