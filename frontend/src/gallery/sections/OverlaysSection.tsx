import { useState } from "react";

import { Button } from "../../components/Button";
import { ContextMenu } from "../../components/ContextMenu";
import { Dialog } from "../../components/Dialog";
import { SidePanel } from "../../components/SidePanel";
import { Stepper } from "../../components/Stepper";
import type { ToastMessage } from "../../components/Toast";
import { ToastViewport } from "../../components/ToastViewport";
import { TextArea } from "../../components/TextArea";
import { TextField } from "../../components/TextField";
import { Tooltip } from "../../components/Tooltip";
import { gallery } from "../../i18n/gallery";
import { menuEntries, sample, steps, toasts } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;

/** Tooltip, context menu, toasts, dialog and side panel, opened on demand. */
export function OverlaysSection() {
  const [menu, setMenu] = useState<{ x: number; y: number }>();
  const [shown, setShown] = useState<ToastMessage[]>([]);
  const [dialog, setDialog] = useState(false);
  const [panel, setPanel] = useState(false);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const mismatch = confirm !== "" && confirm !== password;
  return (
    <GallerySection id="overlays" title={gallery.section.overlays}>
      <div className={styles.grid}>
        <Specimen label={caption.tooltip}>
          <Tooltip content={sample.tooltip} shortcut={sample.tooltipShortcut}>
            <Button variant="primary">{sample.connect}</Button>
          </Tooltip>
          <Tooltip content={sample.tooltipLong} multiline>
            <Button>{sample.inherited}</Button>
          </Tooltip>
        </Specimen>
        <Specimen label={caption.contextMenu}>
          <Button
            onClick={(event) => {
              const box = event.currentTarget.getBoundingClientRect();
              setMenu({ x: box.left, y: box.bottom });
            }}
          >
            {gallery.open.contextMenu}
          </Button>
        </Specimen>
        <Specimen label={caption.toasts}>
          <Button onClick={() => setShown(toasts)}>{gallery.open.toasts}</Button>
        </Specimen>
        <Specimen label={caption.dialog}>
          <Button onClick={() => setDialog(true)}>{gallery.open.dialog}</Button>
        </Specimen>
        <Specimen label={caption.sidePanel}>
          <Button onClick={() => setPanel(true)}>{gallery.open.sidePanel}</Button>
        </Specimen>
      </div>
      {menu && (
        <ContextMenu
          label="web01"
          entries={menuEntries}
          position={menu}
          onSelect={() => undefined}
          onClose={() => setMenu(undefined)}
        />
      )}
      <ToastViewport
        toasts={shown}
        onDismiss={(id) => setShown((current) => current.filter((toast) => toast.id !== id))}
      />
      <Dialog
        open={dialog}
        onClose={() => setDialog(false)}
        title={sample.dialogTitle}
        subtitle={sample.dialogSubtitle}
        footer={
          <>
            <Button onClick={() => setDialog(false)}>{sample.cancel}</Button>
            <Button variant="primary" disabled={password === "" || mismatch}>
              {sample.set}
            </Button>
          </>
        }
      >
        <TextField
          label={sample.password}
          type="password"
          value={password}
          onChange={(event) => setPassword(event.target.value)}
        />
        <TextField
          label={sample.confirm}
          type="password"
          value={confirm}
          shake
          onChange={(event) => setConfirm(event.target.value)}
          message={
            confirm === ""
              ? undefined
              : mismatch
                ? { tone: "error", text: sample.passwordsDiffer }
                : { tone: "ok", text: sample.passwordsMatch }
          }
        />
        <p className={styles.note}>{sample.dialogText}</p>
      </Dialog>
      <SidePanel open={panel} onClose={() => setPanel(false)} label={sample.editorTitle}>
        <h2 className={styles.sectionTitle}>{sample.editorTitle}</h2>
        <Stepper variant="editor" steps={steps.editorOpen} onSelect={() => undefined} />
        <TextField label={sample.name} defaultValue="web01" />
        <TextArea label={sample.notes} />
        <Button variant="primary" size="wide" onClick={() => setPanel(false)}>
          {sample.set}
        </Button>
      </SidePanel>
    </GallerySection>
  );
}
