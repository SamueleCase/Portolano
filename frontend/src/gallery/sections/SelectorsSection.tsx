import { useState } from "react";

import { Button } from "../../components/Button";
import { CredentialPreview } from "../../components/CredentialPreview";
import type { CredentialMode } from "../../components/credentialPreviewData";
import { CredentialSelector } from "../../components/CredentialSelector";
import type { NodeIconChoice } from "../../components/icons/NodeIcon";
import { IconPicker } from "../../components/IconPicker";
import { NodePicker } from "../../components/NodePicker";
import { gallery } from "../../i18n/gallery";
import { customIcons, pickerNodes, previews, sample } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;

/** The credential selector with every state of its preview, and the pickers. */
export function SelectorsSection() {
  const [selector, setSelector] = useState(false);
  const [mode, setMode] = useState<CredentialMode>("inherit");
  const [iconPicker, setIconPicker] = useState(false);
  const [icon, setIcon] = useState<NodeIconChoice>();
  const [nodePicker, setNodePicker] = useState(false);
  const preview = previews.find((data) => data.mode === mode) ?? { mode: "none" };
  return (
    <GallerySection id="selectors" title={gallery.section.selectors}>
      <div className={styles.grid}>
        <Specimen label={caption.credentialSelector}>
          <Button onClick={() => setSelector(true)}>{gallery.open.credentialSelector}</Button>
        </Specimen>
        <Specimen label={caption.iconPicker}>
          <Button onClick={() => setIconPicker(true)}>{gallery.open.iconPicker}</Button>
        </Specimen>
        <Specimen label={caption.nodePicker}>
          <Button onClick={() => setNodePicker(true)}>{gallery.open.nodePicker}</Button>
        </Specimen>
      </div>
      <div className={styles.grid}>
        {previews.map((data, index) => (
          <Specimen
            key={index}
            label={`${caption.preview} · ${caption.previewStates[index] ?? ""}`}
          >
            <div className={styles.column}>
              <CredentialPreview data={data} backend="keychain" />
            </div>
          </Specimen>
        ))}
      </div>
      <CredentialSelector
        open={selector}
        onClose={() => setSelector(false)}
        onDone={() => setSelector(false)}
        mode={mode}
        onModeChange={setMode}
        preview={preview}
        backend="keychain"
      />
      <IconPicker
        open={iconPicker}
        onClose={() => setIconPicker(false)}
        value={icon}
        onUse={(choice) => {
          setIcon(choice);
          setIconPicker(false);
        }}
        customIcons={customIcons}
        onChooseImage={() => undefined}
      />
      <NodePicker
        open={nodePicker}
        onClose={() => setNodePicker(false)}
        onChoose={() => setNodePicker(false)}
        title={sample.pickerTitle}
        filterLabel={sample.pickerFilter}
        nodes={pickerNodes}
        note={sample.pickerNote}
      />
    </GallerySection>
  );
}
