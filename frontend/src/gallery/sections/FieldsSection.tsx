import { useState } from "react";

import { ColorPicker } from "../../components/ColorPicker";
import { KeyValueEditor } from "../../components/KeyValueEditor";
import { Select } from "../../components/Select";
import { TagInput } from "../../components/TagInput";
import { TextArea } from "../../components/TextArea";
import { TextField } from "../../components/TextField";
import { gallery } from "../../i18n/gallery";
import { properties, sample, sshOptions } from "../fixtures";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

const caption = gallery.caption;
const depths = [
  { value: "16", label: "16 bit" },
  { value: "24", label: "24 bit" },
  { value: "32", label: "32 bit" },
] as const;
const ignore = () => undefined;

/** Text fields, text area, select, tags, key-value editors and colors. */
export function FieldsSection() {
  const [depth, setDepth] = useState<"16" | "24" | "32">("32");
  const [tags, setTags] = useState(["web", "produzione"]);
  const [swatchColor, setSwatchColor] = useState<string | null>("#7FA6D9");
  const [noColor, setNoColor] = useState<string | null>(null);
  const [customColor, setCustomColor] = useState<string | null>("#3E8E9C");
  return (
    <GallerySection id="fields" title={gallery.section.fields}>
      <div className={styles.grid}>
        <Specimen label={caption.fieldRest}>
          <TextField label={sample.name} placeholder={sample.host} />
        </Specimen>
        <Specimen label={caption.fieldMono}>
          <TextField label={sample.host} mono defaultValue="web01.example.com" />
        </Specimen>
        <Specimen label={caption.fieldError}>
          <TextField
            label={sample.port}
            mono
            defaultValue="70000"
            message={{ tone: "error", text: sample.portError }}
          />
        </Specimen>
        <Specimen label={caption.fieldConfirmed}>
          <TextField
            label={sample.confirm}
            type="password"
            defaultValue="campione"
            message={{ tone: "ok", text: sample.passwordsMatch }}
          />
        </Specimen>
        <Specimen label={caption.fieldPassword}>
          <TextField label={sample.password} type="password" defaultValue="campione" />
        </Specimen>
        <Specimen label={caption.fieldDisabled}>
          <TextField label={sample.description} disabled defaultValue="—" />
        </Specimen>
        <Specimen label={caption.fieldSearch}>
          <TextField label={sample.search} variant="search" placeholder={sample.search} />
        </Specimen>
        <Specimen label={caption.fieldNumeric}>
          <TextField label={sample.keepalive} variant="numeric" mono defaultValue="30" />
        </Specimen>
        <Specimen label={caption.textArea}>
          <TextArea label={sample.notes} defaultValue="Replica in sola lettura." />
        </Specimen>
        <Specimen label={caption.select}>
          <Select label={sample.depth} options={depths} value={depth} onChange={setDepth} />
        </Specimen>
        <Specimen label={caption.tags}>
          <TagInput label={sample.tags} tags={tags} onChange={setTags} />
        </Specimen>
      </div>
      <div className={styles.grid}>
        <Specimen label={caption.colorSwatch}>
          <ColorPicker value={swatchColor} onChange={setSwatchColor} />
        </Specimen>
        <Specimen label={caption.colorNone}>
          <ColorPicker value={noColor} onChange={setNoColor} />
        </Specimen>
        <Specimen label={caption.colorCustom}>
          <ColorPicker value={customColor} onChange={setCustomColor} />
        </Specimen>
      </div>
      <div className={styles.wide}>
        <Specimen label={sample.properties}>
          <div className={styles.column}>
            <KeyValueEditor
              label={sample.properties}
              variant="properties"
              entries={properties}
              backend="keychain"
              addLabel={sample.addProperty}
              onChange={ignore}
              onReplaceSecret={ignore}
              onRemove={ignore}
              onAdd={ignore}
            />
          </div>
        </Specimen>
        <Specimen label={sample.options}>
          <div className={styles.column}>
            <KeyValueEditor
              label={sample.options}
              variant="options"
              entries={sshOptions}
              backend="keychain"
              addLabel={sample.addOption}
              onChange={ignore}
              onReplaceSecret={ignore}
              onRemove={ignore}
              onAdd={ignore}
            />
          </div>
        </Specimen>
      </div>
    </GallerySection>
  );
}
