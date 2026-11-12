import { type KeyboardEvent, useRef } from "react";

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
  const buttons = useRef(new Map<CredentialMode, HTMLButtonElement>());
  const chosen = useRef<HTMLButtonElement | null>(null);

  const keyDown = (event: KeyboardEvent, index: number) => {
    const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[event.key];
    if (step === undefined) return;
    const next = modes[(index + step + modes.length) % modes.length];
    if (!next) return;
    event.preventDefault();
    onModeChange(next);
    buttons.current.get(next)?.focus();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      label={t.credentialSelector.label}
      width="selector"
      initialFocus={chosen}
      footer={
        <Button variant="primary" onClick={onDone}>
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
                  if (element) buttons.current.set(option, element);
                  else buttons.current.delete(option);
                  if (selected) chosen.current = element;
                }}
                type="button"
                role="radio"
                aria-checked={selected}
                tabIndex={selected ? 0 : -1}
                className={classNames(styles.mode, selected && styles.selected)}
                onClick={() => onModeChange(option)}
                onKeyDown={(event) => keyDown(event, index)}
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
