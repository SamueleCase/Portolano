import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import { errorMessage, useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import { DEFAULT_PORTS } from "./editorKinds";

/**
 * Host and port of a connection: an empty port is the default of the
 * protocol, saved as no port at all (NOTE.md).
 */
export function ConnectionTarget({ protocol }: { protocol: "ssh" | "rdp" }) {
  const t = texts.editor.connection;
  const { values, set, errorOf } = useEditorFields();
  return (
    <div className={styles.hostPort}>
      <TextField
        label={t.host}
        mono
        value={values.host}
        message={errorMessage(errorOf("host"))}
        onChange={(event) => set("host", event.target.value)}
      />
      <TextField
        label={t.port}
        mono
        inputMode="numeric"
        placeholder={String(DEFAULT_PORTS[protocol])}
        value={values.port}
        message={errorMessage(errorOf("port"))}
        onChange={(event) => set("port", event.target.value)}
      />
    </div>
  );
}
