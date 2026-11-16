import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import { type CustomIconOption, IconPicker } from "../../components/IconPicker";
import type { IconChoice, IconView } from "../../bindings";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { useAddIcon, useChooseIconFile } from "../../ipc/mutations";
import { customIconsQuery } from "../../ipc/queries";
import styles from "./Dialogs.module.css";
import { drawnIcon, iconFileName, savedIcon } from "./iconChoice";

interface IconChooserProps {
  open: boolean;
  /** The icon of the node now. */
  value: IconChoice | null;
  onUse: (choice: IconChoice) => void;
  onClose: () => void;
}

/** The length of the hash that names an imported image, which has no name of its own. */
const SHORT_HASH = 8;

function options(stored: readonly IconView[]): CustomIconOption[] {
  return stored.flatMap((icon) =>
    icon.kind === "custom"
      ? [{ fileName: iconFileName(icon), label: icon.sha256.slice(0, SHORT_HASH) }]
      : [],
  );
}

/**
 * The icon picker over the open document (inventario §5.7): the builtin set,
 * the images stored with the document (`custom_icons`), or a new image
 * chosen with the native dialog of Rust (`choose_icon_file` → `add_icon`),
 * which is used at once. An image that cannot be stored is said over the
 * picker, which stays open to choose again.
 */
export function IconChooser({ open, value, onUse, onClose }: IconChooserProps) {
  const t = texts.dialogs.iconError;
  const { data: stored = [] } = useQuery({ ...customIconsQuery, enabled: open });
  const chooseFile = useChooseIconFile();
  const addIcon = useAddIcon();
  const [error, setError] = useState<string>();

  const chooseImage = async () => {
    const chosen = await chooseFile.mutateAsync();
    if (chosen === null) return;
    const icon = await addIcon.mutateAsync();
    if (icon.kind === "custom") onUse({ kind: "custom", sha256: icon.sha256, mime: icon.mime });
  };

  return (
    <>
      <IconPicker
        open={open}
        onClose={onClose}
        value={drawnIcon(value, stored)}
        customIcons={options(stored)}
        onUse={(choice) => {
          const icon = savedIcon(choice, stored);
          if (icon) onUse(icon);
        }}
        onChooseImage={() =>
          void chooseImage().catch((failure: unknown) => setError(presentFailure(failure).message))
        }
      />
      <Dialog
        open={open && error !== undefined}
        onClose={() => setError(undefined)}
        title={t.title}
        width="small"
        footer={
          <Button variant="primary" onClick={() => setError(undefined)}>
            {t.close}
          </Button>
        }
      >
        <p className={styles.hint} role="alert">
          {error}
        </p>
      </Dialog>
    </>
  );
}
