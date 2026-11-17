import { GroupHeader } from "../../components/GroupHeader";
import { texts } from "../../i18n/it";
import { CredentialSetting } from "./CredentialSetting";
import { useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";

/** Step 2 · Credenziale predefinita of a folder (inventario §5.5). */
export function FolderStep() {
  const t = texts.editor.folder;
  const { values, set, errorOf } = useEditorFields();
  return (
    <section className={styles.step} aria-label={texts.editor.steps.folderCredential}>
      <GroupHeader>{texts.editor.steps.folderCredential}</GroupHeader>
      <CredentialSetting
        label={t.credential}
        value={values.credential}
        onChange={(credential) => set("credential", credential)}
        ownPassword
        error={errorOf("credential")}
      />
      <p className={styles.hint}>{t.note}</p>
    </section>
  );
}
