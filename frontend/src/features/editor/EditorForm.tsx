import { zodResolver } from "@hookform/resolvers/zod";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { type FieldErrors, useForm } from "react-hook-form";

import { Button } from "../../components/Button";
import { IconTile } from "../../components/IconTile";
import { type Step, Stepper } from "../../components/Stepper";
import { visibleText } from "../../components/visibleText";
import type { NodeDetailsView_Serialize } from "../../bindings";
import { texts } from "../../i18n/it";
import { customIconsQuery } from "../../ipc/queries";
import { drawnIcon } from "../dialogs/iconChoice";
import { readablePath } from "../dialogs/nodePaths";
import { SecretDialog } from "../dialogs/SecretDialog";
import { useVaultState } from "../dialogs/vault";
import { CredentialStep } from "./CredentialStep";
import { type EditorFields, EditorFieldsContext } from "./editorContext";
import styles from "./Editor.module.css";
import { secondStepLabel } from "./editorKinds";
import { EditorNotices } from "./EditorNotices";
import { editorSchema, missingVaultKey } from "./editorSchema";
import type { EditorValues } from "./editorValues";
import { FolderStep } from "./FolderStep";
import { GeneralStep } from "./GeneralStep";
import { RdpStep } from "./RdpStep";
import { secretSlots, slotInput, type SlotKey } from "./secretSlots";
import { type ErrorTarget, stepOf } from "./serverErrors";
import { SshStep } from "./SshStep";
import { StagedSecrets } from "./stagedSecrets";
import { type SavedNode, useEditorSave } from "./useEditorSave";
import { useExternalChanges } from "./useExternalChanges";

interface EditorFormProps {
  initial: EditorValues;
  /** The detail of the node being edited, kept fresh; `undefined` for a new node. */
  details: NodeDetailsView_Serialize | undefined;
  onClose: () => void;
  onSaved: (node: SavedNode) => void;
}

const STEP_BODIES = { ssh: SshStep, rdp: RdpStep, folder: FolderStep, credential: CredentialStep };

/** The first field with an error of the schema, to show its step. */
function firstError(errors: FieldErrors<EditorValues>): ErrorTarget | undefined {
  return Object.keys(errors).find((field): field is ErrorTarget => field !== "root");
}

/**
 * The editor of one node in two steps (inventario §5.5): "Salva" on both
 * steps, enabled once something changed; secrets staged until "Salva".
 */
export function EditorForm({ initial, details, onClose, onSaved }: EditorFormProps) {
  const t = texts.editor;
  const form = useForm<EditorValues>({
    defaultValues: initial,
    resolver: zodResolver(editorSchema),
  });
  // The whole form, read at every render: the steps show what depends on
  // other fields. The app does not run the React Compiler, whose
  // memoization this rule protects (`watch` re-renders on every change).
  // eslint-disable-next-line react-hooks/incompatible-library
  const values = form.watch();
  const { backend } = useVaultState();
  const { data: storedIcons = [] } = useQuery(customIconsQuery);
  const { base, notice } = useExternalChanges(form, initial, details);
  const staged = useRef<StagedSecrets>(null);
  staged.current ??= new StagedSecrets();
  const holder = staged.current;
  const { save, saving, dialog } = useEditorSave(holder);
  const [step, setStep] = useState<0 | 1>(0);
  const [serverErrors, setServerErrors] = useState<Partial<Record<ErrorTarget, string>>>({});
  const [secretsLost, setSecretsLost] = useState(false);
  const [asking, setAsking] = useState<SlotKey>();

  // Leaving the editor in any way drops what is staged.
  useEffect(() => () => holder.clear(), [holder]);

  const secrets = secretSlots(details?.secrets ?? [], values.properties, values.secrets);
  const fields: EditorFields = {
    values,
    set: (field, value) => {
      // The form is typed by field: `setValue` cannot tie a generic key to its value.
      form.setValue(field as "name", value as EditorValues["name"], {
        shouldDirty: true,
        shouldValidate: form.formState.isSubmitted,
      });
      setServerErrors((current) => ({ ...current, [field]: undefined }));
    },
    errorOf: (target) => {
      const checked =
        target === "form" ? undefined : form.formState.errors[target as keyof EditorValues];
      return typeof checked?.message === "string" ? checked.message : serverErrors[target];
    },
    secrets,
    askSecret: setAsking,
    clearSecret: (key) => {
      holder.drop(key);
      const marks = { ...form.getValues("secrets") };
      if (secrets.stored(key)) marks[key] = "clear";
      else delete marks[key];
      fields.set("secrets", marks);
    },
    backend,
    nodeId: details?.id,
  };

  const close = () => {
    holder.clear();
    onClose();
  };

  const submit = form.handleSubmit(
    async (valid) => {
      const storedKey = secrets.stored("privateKey");
      if (missingVaultKey(valid, storedKey)) {
        setServerErrors({ privateKey: t.errors.vaultKey });
        setStep(1);
        return;
      }
      setServerErrors({});
      setSecretsLost(false);
      const result = await save({ values: valid, base, details });
      if (result.status === "saved") {
        onSaved(result.node);
        return;
      }
      if (result.status === "cancelled") return;
      setServerErrors({ [result.error.target]: result.error.message });
      if (result.error.target !== "form") setStep(stepOf(result.error.target));
      if (result.secretsLost) {
        setSecretsLost(true);
        const marks = Object.entries(form.getValues("secrets")).filter(
          ([, mark]) => mark !== "set",
        );
        fields.set("secrets", Object.fromEntries(marks));
      }
    },
    (errors) => {
      const field = firstError(errors);
      if (field) setStep(stepOf(field));
    },
  );

  const steps: Step[] = [
    { label: t.steps.general, state: step === 0 ? "current" : "done" },
    { label: secondStepLabel(values.kind), state: step === 1 ? "current" : "upcoming" },
  ];
  const subtitle = (details ? t.subtitles.edit : t.subtitles.add)[values.kind];
  const Body = step === 0 ? GeneralStep : STEP_BODIES[values.kind];
  const askingSlot = asking === undefined ? undefined : slotInput(asking, values.properties);

  return (
    <EditorFieldsContext.Provider value={fields}>
      <header className={styles.header}>
        <IconTile
          kind={values.kind}
          icon={drawnIcon(values.icon, storedIcons)}
          color={values.color}
          size="editor"
        />
        <div className={styles.heading}>
          <p className={styles.subtitle}>{subtitle}</p>
          <h2 className={styles.title}>{visibleText(values.name || t.untitled)}</h2>
        </div>
        <Button onClick={close}>{t.cancel}</Button>
      </header>
      <Stepper variant="editor" steps={steps} onSelect={(index) => setStep(index === 0 ? 0 : 1)} />
      <form
        className={styles.form}
        noValidate
        onSubmit={(event) => {
          event.preventDefault();
          // Enter in a field submits too: nothing to save while nothing changed.
          if (form.formState.isDirty) void submit();
        }}
      >
        <Body />
        <EditorNotices external={notice} error={serverErrors.form} secretsLost={secretsLost} />
        <footer className={styles.footer}>
          {step === 1 && <Button onClick={() => setStep(0)}>{t.back}</Button>}
          <span className={styles.spacer} />
          {step === 0 && <Button onClick={() => setStep(1)}>{t.next}</Button>}
          <Button
            variant="primary"
            size="wide"
            type="submit"
            disabled={!form.formState.isDirty}
            loading={saving}
          >
            {t.save}
          </Button>
        </footer>
      </form>
      <SecretDialog
        open={askingSlot !== undefined}
        slot={askingSlot ?? { kind: "password" }}
        nodeLabel={details ? readablePath(details.path) : values.name || t.untitled}
        replacing={asking !== undefined && ["present", "staged"].includes(secrets.state(asking))}
        mode="stage"
        backend={backend}
        onClose={() => setAsking(undefined)}
        onConfirm={(value) => {
          if (asking === undefined) return;
          holder.stage(asking, value);
          fields.set("secrets", { ...form.getValues("secrets"), [asking]: "set" });
          setAsking(undefined);
        }}
      />
      {dialog}
    </EditorFieldsContext.Provider>
  );
}
