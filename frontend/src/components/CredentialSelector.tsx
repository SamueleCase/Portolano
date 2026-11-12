import { useRef } from "react";

import { texts } from "../i18n/it";
import { Button } from "./Button";
import { classNames } from "./classNames";
import { CredentialPreview } from "./CredentialPreview";
import {
  type CredentialMode,
  credentialModes,
  type CredentialPreviewActions,
  type CredentialPreviewData,
} from "./credentialPreviewData";
import styles from "./CredentialSelector.module.css";
import { Dialog } from "./Dialog";
import type { VaultBackend } from "./SecretBadge";
import { useRovingRadio } from "./useRovingRadio";

interface CredentialSelectorProps {
  open: boolean;
  onClose: () => void;
  /** "Fatto": the chosen mode and its details apply. */
  onDone: () => void;
  mode: CredentialMode;
  onModeChange: (mode: CredentialMode) => void;
  /** What the chosen mode resolves to, from the backend. */
  preview: CredentialPreviewData;
  backend: VaultBackend;
  actions?: CredentialPreviewActions | undefined;
  /** The modes offered, for instance without "Chiedi all'avvio" for a folder. */
  modes?: readonly CredentialMode[];
}

/**
 * The credential selector as a modal dialog (ambiguity 6, inventario §4.24):
 * the modes on the left as a radio group, "Da dove arriverà" on the right.
 */
export function CredentialSelector({
  open,
  onClose,
  onDone,
  mode,
  onModeChange,
  preview,
  backend,
  actions,
  modes = credentialModes,
}: CredentialSelectorProps) {
  const t = texts.components;
  const chosen = useRef<HTMLButtonElement | null>(null);
  const radio = useRovingRadio({
    count: modes.length,
    selected: modes.indexOf(mode),
    onMove: (index) => {
      const next = modes[index];
      if (next) onModeChange(next);
    },
  });

  return (
    <Dialog
      open={open}
      onClose={onClose}
      label={t.credentialSelector.label}
      width="selector"
      initialFocus={chosen}
      onSubmit={onDone}
      footer={
        <Button variant="primary" type="submit">
          {t.credentialSelector.done}
        </Button>
      }
    >
      <div className={styles.layout}>
        <div role="radiogroup" aria-label={t.credentialSelector.modes} className={styles.modes}>
          {modes.map((option, index) => {
            const selected = option === mode;
            return (
              <button
                key={option}
                ref={(element) => {
                  radio.register(index)(element);
                  if (selected) chosen.current = element;
                }}
                type="button"
                role="radio"
                aria-checked={selected}
                tabIndex={radio.tabIndexOf(index)}
                className={classNames(styles.mode, selected && styles.selected)}
                onClick={() => onModeChange(option)}
                onKeyDown={(event) => radio.keyDown(event, index)}
              >
                <span className={styles.title}>{t.credentialModes[option].title}</span>
                <span className={styles.subtitle}>{t.credentialModes[option].subtitle}</span>
              </button>
            );
          })}
        </div>
        <CredentialPreview data={preview} backend={backend} actions={actions} />
      </div>
    </Dialog>
  );
}
