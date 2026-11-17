import type { CredentialMode } from "../../components/credentialPreviewData";
import { GroupHeader } from "../../components/GroupHeader";
import { SegmentedControl } from "../../components/SegmentedControl";
import { Select } from "../../components/Select";
import { TextField } from "../../components/TextField";
import { ToggleRow } from "../../components/ToggleRow";
import { texts } from "../../i18n/it";
import { ConnectionTarget } from "./ConnectionTarget";
import { CredentialSetting } from "./CredentialSetting";
import { errorMessage, useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import type { EditorValues } from "./editorValues";

/**
 * The modes of the credential of a gateway: its inline password has no
 * place in the staged secrets, and "Chiedi all'avvio" asks only the login of
 * the destination (DEC-038.4).
 */
const GATEWAY_MODES: readonly CredentialMode[] = ["inherit", "none", "linked", "byName"];

const COLOR_DEPTHS = [
  "",
  "15",
  "16",
  "24",
  "32",
] as const satisfies readonly EditorValues["colorDepth"][];

/** Step 2 · Connessione of an RDP connection (inventario §5.5). */
export function RdpStep() {
  const t = texts.editor.rdp;
  const { values, set, errorOf } = useEditorFields();

  return (
    <section className={styles.step} aria-label={texts.editor.steps.connection}>
      <GroupHeader>{texts.editor.steps.connection}</GroupHeader>
      <ConnectionTarget protocol="rdp" />
      <CredentialSetting
        label={texts.editor.connection.credential}
        value={values.credential}
        onChange={(credential) => set("credential", credential)}
        ownPassword
        error={errorOf("credential")}
      />
      <GroupHeader>{t.screen}</GroupHeader>
      <div className={styles.group}>
        <span className={styles.fieldLabel}>{t.resolution}</span>
        <SegmentedControl
          label={t.resolution}
          value={values.resolution}
          onChange={(resolution) => set("resolution", resolution)}
          options={[
            { value: "fit", label: t.resolutions.fit },
            { value: "fullscreen", label: t.resolutions.fullscreen },
            { value: "fixed", label: t.resolutions.fixed },
          ]}
        />
        {values.resolution === "fixed" && (
          <div className={styles.size}>
            <TextField
              label={t.width}
              mono
              variant="numeric"
              inputMode="numeric"
              value={values.width}
              message={errorMessage(errorOf("width"))}
              onChange={(event) => set("width", event.target.value)}
            />
            <span aria-hidden="true">×</span>
            <TextField
              label={t.height}
              mono
              variant="numeric"
              inputMode="numeric"
              value={values.height}
              onChange={(event) => set("height", event.target.value)}
            />
          </div>
        )}
      </div>
      <div className={styles.pair}>
        <div className={styles.group}>
          <span className={styles.fieldLabel}>{t.scale}</span>
          <SegmentedControl
            label={t.scale}
            value={values.scale}
            onChange={(scale) => set("scale", scale)}
            options={[
              { value: "100", label: "100%" },
              { value: "140", label: "140%" },
              { value: "180", label: "180%" },
            ]}
          />
        </div>
        <Select
          label={t.colorDepth}
          value={values.colorDepth}
          onChange={(colorDepth) => set("colorDepth", colorDepth)}
          options={COLOR_DEPTHS.map((depth) => ({ value: depth, label: t.colorDepths[depth] }))}
        />
      </div>
      <GroupHeader>{t.gateway}</GroupHeader>
      <div className={styles.hostPort}>
        <TextField
          label={t.gatewayHost}
          mono
          value={values.gatewayHost}
          message={errorMessage(errorOf("gatewayHost"))}
          onChange={(event) => set("gatewayHost", event.target.value)}
        />
        <TextField
          label={t.gatewayPort}
          mono
          inputMode="numeric"
          value={values.gatewayPort}
          message={errorMessage(errorOf("gatewayPort"))}
          onChange={(event) => set("gatewayPort", event.target.value)}
        />
      </div>
      {values.gatewayHost.trim() !== "" && (
        <CredentialSetting
          label={t.gatewayCredential}
          value={values.gatewayCredential}
          modes={GATEWAY_MODES}
          ownPassword={false}
          onChange={(credential) => {
            if (credential.mode !== "inline" && credential.mode !== "prompt") {
              set("gatewayCredential", credential);
            }
          }}
        />
      )}
      <GroupHeader>{t.session}</GroupHeader>
      <div className={styles.toggles}>
        <ToggleRow
          label={t.multimon}
          checked={values.multimon}
          onChange={(on) => set("multimon", on)}
        />
        <ToggleRow label={t.admin} checked={values.admin} onChange={(on) => set("admin", on)} />
        <ToggleRow
          label={t.clipboard}
          checked={values.clipboard}
          onChange={(on) => set("clipboard", on)}
        />
      </div>
      <div className={styles.pair}>
        <TextField
          label={t.wolMac}
          mono
          value={values.wolMac}
          message={errorMessage(errorOf("wolMac"))}
          onChange={(event) => set("wolMac", event.target.value)}
        />
        <div className={styles.group}>
          <span className={styles.fieldLabel}>{t.client}</span>
          <SegmentedControl
            label={t.client}
            value={values.client}
            onChange={(client) => set("client", client)}
            options={[
              { value: "auto", label: t.clients.auto },
              { value: "freerdp", label: t.clients.freerdp },
              { value: "windows-app", label: t.clients["windows-app"] },
            ]}
          />
        </div>
      </div>
    </section>
  );
}
