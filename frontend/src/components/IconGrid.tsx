import { type KeyboardEvent, type ReactNode, useRef } from "react";

import { classNames } from "./classNames";
import styles from "./IconGrid.module.css";
import type { NodeIconChoice } from "./icons/NodeIcon";

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
  const buttons = useRef(new Map<string, HTMLButtonElement>());
  const focusIndex = Math.max(
    0,
    tiles.findIndex((tile) => tile.id === selectedId),
  );

  const keyDown = (event: KeyboardEvent, index: number) => {
    const step = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: COLUMNS, ArrowUp: -COLUMNS }[event.key];
    if (step === undefined) return;
    const next = tiles[index + step];
    if (!next) return;
    event.preventDefault();
    onSelect(next);
    buttons.current.get(next.id)?.focus();
  };

  return (
    <div role="radiogroup" aria-label={label} className={styles.grid}>
      {tiles.map((tile, index) => {
        const selected = tile.id === selectedId;
        return (
          <button
            key={tile.id}
            ref={(element) => {
              if (element) buttons.current.set(tile.id, element);
              else buttons.current.delete(tile.id);
            }}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={tile.label}
            title={tile.label}
            tabIndex={index === focusIndex ? 0 : -1}
            className={classNames(styles.tile, selected && styles.selected)}
            onClick={() => onSelect(tile)}
            onKeyDown={(event) => keyDown(event, index)}
          >
            {tile.icon}
          </button>
        );
      })}
    </div>
  );
}
