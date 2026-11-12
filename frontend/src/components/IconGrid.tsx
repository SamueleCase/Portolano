import type { ReactNode } from "react";

import { classNames } from "./classNames";
import styles from "./IconGrid.module.css";
import type { NodeIconChoice } from "./icons/NodeIcon";
import { useRovingRadio } from "./useRovingRadio";

/** A tile of the icon grid. */
export interface IconGridTile {
  id: string;
  /** The accessible name: the Italian label or the imported name. */
  label: string;
  icon: ReactNode;
  choice: NodeIconChoice;
}

interface IconGridProps {
  label: string;
  tiles: readonly IconGridTile[];
  selectedId: string | undefined;
  onSelect: (tile: IconGridTile) => void;
}

/** Tiles per row (inventario §5.7). */
const COLUMNS = 7;

/**
 * The grid of icons, a radio group of seven columns: one tab stop, the
 * arrows move in two dimensions and choose.
 */
export function IconGrid({ label, tiles, selectedId, onSelect }: IconGridProps) {
  const radio = useRovingRadio({
    count: tiles.length,
    selected: tiles.findIndex((tile) => tile.id === selectedId),
    columns: COLUMNS,
    onMove: (index) => {
      const tile = tiles[index];
      if (tile) onSelect(tile);
    },
  });

  return (
    <div role="radiogroup" aria-label={label} className={styles.grid}>
      {tiles.map((tile, index) => {
        const selected = tile.id === selectedId;
        return (
          <button
            key={tile.id}
            ref={radio.register(index)}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={tile.label}
            title={tile.label}
            tabIndex={radio.tabIndexOf(index)}
            className={classNames(styles.tile, selected && styles.selected)}
            onClick={() => onSelect(tile)}
            onKeyDown={(event) => radio.keyDown(event, index)}
          >
            {tile.icon}
          </button>
        );
      })}
    </div>
  );
}
