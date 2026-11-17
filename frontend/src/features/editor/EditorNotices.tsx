import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import styles from "./Editor.module.css";
import type { EditableField } from "./editorValues";
import type { ExternalNotice } from "./useExternalChanges";

/** The label of a field in the warning of the fields changed on both sides. */
const FIELD_LABELS: Partial<Record<EditableField, string>> = {
  name: texts.editor.general.name,
  parent: texts.editor.general.position,
  description: texts.editor.general.description,
  notes: texts.editor.general.notes,
  icon: texts.editor.general.icon,
  color: texts.components.colorPicker.label,
  favorite: texts.components.favorite,
  tags: texts.editor.general.tags,
  properties: texts.editor.general.properties,
  host: texts.editor.connection.host,
  port: texts.editor.connection.port,
  credential: texts.editor.connection.credential,
  jumpHosts: texts.editor.ssh.jumps,
  options: texts.editor.ssh.options,
  keepalive: texts.editor.ssh.keepalive,
  gatewayHost: texts.editor.rdp.gatewayHost,
  gatewayCredential: texts.editor.rdp.gatewayCredential,
  username: texts.editor.credential.user,
  domain: texts.editor.credential.domain,
  keyFile: texts.editor.credential.keyFile,
};

function fieldLabel(field: EditableField): string {
  return FIELD_LABELS[field] ?? texts.editor.steps.connection;
}

interface EditorNoticesProps {
  external: ExternalNotice;
  /** The error of the backend that has no field of its own. */
  error?: string | undefined;
  /** The last "Salva" failed with staged secrets, which are gone. */
  secretsLost: boolean;
}

/** What the editor has to tell above its buttons: changes from outside, errors. */
export function EditorNotices({ external, error, secretsLost }: EditorNoticesProps) {
  const t = texts.editor;
  const conflicts =
    external?.kind === "reapplied" && external.conflicts.length > 0
      ? interpolate(t.external.conflicts, {
          fields: [...new Set(external.conflicts.map(fieldLabel))].join(", "),
        })
      : undefined;
  return (
    <>
      {external && (
        <p className={styles.notice} role="status">
          {external.kind === "modified" ? t.external.modified : t.external.reapplied}
          {conflicts && ` ${conflicts}`}
        </p>
      )}
      {(error !== undefined || secretsLost) && (
        <div className={styles.failure} role="alert">
          {error && <p>{error}</p>}
          {secretsLost && <p>{t.errors.secretsLost}</p>}
        </div>
      )}
    </>
  );
}
