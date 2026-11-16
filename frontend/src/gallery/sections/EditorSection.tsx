import { QueryClientProvider, useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "../../components/Button";
import type { CredentialChoice_Deserialize } from "../../bindings";
import { CreateVaultDialog } from "../../features/dialogs/CreateVaultDialog";
import { CredentialChooser } from "../../features/dialogs/CredentialChooser";
import { IconChooser } from "../../features/dialogs/IconChooser";
import { SetSecretDialog } from "../../features/dialogs/SetSecretDialog";
import { TreeNodePicker } from "../../features/dialogs/TreeNodePicker";
import { indexTree } from "../../features/dialogs/treeIndex";
import { UnlockVaultDialog } from "../../features/dialogs/UnlockVaultDialog";
import { useLoginPrompt } from "../../features/dialogs/useLoginPrompt";
import { type EditorRequest, NodeEditor } from "../../features/editor/NodeEditor";
import { PreferencesDialog } from "../../features/preferences/PreferencesDialog";
import { interpolate } from "../../i18n/format";
import { editorGallery } from "../../i18n/editor";
import { gallery } from "../../i18n/gallery";
import { createQueryClient } from "../../ipc/queryClient";
import { treeQuery } from "../../ipc/queries";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

type Dialog = "password" | "unlock" | "create" | "credential" | "icon" | "jumpHost" | "preferences";

// The section has its own cache: in the e2e build the fake backend answers it.
const queryClient = createQueryClient();

/**
 * The editor, the dialogs and the preferences of UI-WP5 on the fake backend
 * (`npm run e2e`), with the sample document: nodes are found by name.
 */
export function EditorSection() {
  return (
    <QueryClientProvider client={queryClient}>
      <GallerySection id="editor" title={gallery.section.editor}>
        <EditorSpecimens />
      </GallerySection>
    </QueryClientProvider>
  );
}

function EditorSpecimens() {
  const t = editorGallery.open;
  const { data: tree } = useQuery(treeQuery);
  const entries = tree ? indexTree(tree) : [];
  const idOf = (name: string) => entries.find((entry) => entry.node.name === name)?.node.id;
  const [request, setRequest] = useState<EditorRequest | null>(null);
  const [dialog, setDialog] = useState<Dialog>();
  const [credential, setCredential] = useState<CredentialChoice_Deserialize>({ mode: "inherit" });
  const [result, setResult] = useState("");
  const login = useLoginPrompt();
  const close = () => setDialog(undefined);

  const edit = (name: string) => {
    const id = idOf(name);
    if (id) setRequest({ mode: "edit", id });
  };
  const add = (kind: "ssh" | "rdp" | "folder" | "credential", parent: string | null) =>
    setRequest({ mode: "add", kind, parent: parent === null ? null : (idOf(parent) ?? null) });
  const admin = idOf("Admin ACME");
  const web01 = idOf("web01");

  return (
    <>
      <div className={styles.grid}>
        <Specimen label={editorGallery.editor}>
          <div className={styles.column}>
            <Button onClick={() => add("ssh", "ACME")}>{t.newSsh}</Button>
            <Button onClick={() => add("rdp", "ACME")}>{t.newRdp}</Button>
            <Button onClick={() => add("folder", null)}>{t.newFolder}</Button>
            <Button onClick={() => add("credential", "Rete")}>{t.newCredential}</Button>
            <Button onClick={() => edit("web01")}>{t.editSsh}</Button>
            <Button onClick={() => edit("ts01")}>{t.editRdp}</Button>
            <Button onClick={() => edit("ACME")}>{t.editFolder}</Button>
            <Button onClick={() => edit("deploy")}>{t.editCredential}</Button>
          </div>
        </Specimen>
        <Specimen label={editorGallery.dialogs}>
          <div className={styles.column}>
            <Button onClick={() => setDialog("password")}>{t.password}</Button>
            <Button onClick={() => setDialog("unlock")}>{t.unlock}</Button>
            <Button onClick={() => setDialog("create")}>{t.create}</Button>
            <Button
              onClick={() => {
                const id = idOf("kiosk");
                if (!id) return;
                void login
                  .askLogin(id)
                  .then((answer) =>
                    setResult(
                      answer === null
                        ? editorGallery.cancelled
                        : answer.login
                          ? interpolate(editorGallery.login, { user: answer.login.username ?? "" })
                          : editorGallery.noLogin,
                    ),
                  );
              }}
            >
              {t.login}
            </Button>
            <Button onClick={() => setDialog("credential")}>{t.credential}</Button>
            <Button onClick={() => setDialog("icon")}>{t.icon}</Button>
            <Button onClick={() => setDialog("jumpHost")}>{t.jumpHost}</Button>
            <Button onClick={() => setDialog("preferences")}>{t.preferences}</Button>
          </div>
        </Specimen>
      </div>
      <p role="status">{result}</p>
      <NodeEditor
        request={request}
        onClose={() => setRequest(null)}
        onSaved={(node) => {
          setRequest(null);
          setResult(interpolate(editorGallery.saved, { path: node.path }));
        }}
      />
      <SetSecretDialog
        request={
          dialog === "password" && admin
            ? {
                nodeId: admin,
                nodePath: "/Clienti/Admin ACME",
                slot: { kind: "password" },
                replacing: true,
              }
            : null
        }
        onClose={close}
      />
      <UnlockVaultDialog open={dialog === "unlock"} onClose={close} onUnlocked={close} />
      <CreateVaultDialog open={dialog === "create"} onClose={close} onCreated={close} />
      {login.dialog}
      <CredentialChooser
        open={dialog === "credential"}
        nodeName="web01"
        parent={idOf("ACME") ?? null}
        value={credential}
        passwordState="absent"
        // The inline password of the selector is the editor's: none here.
        onSetPassword={() => undefined}
        onClose={close}
        onDone={(choice) => {
          setCredential(choice);
          close();
        }}
      />
      <IconChooser open={dialog === "icon"} value={null} onClose={close} onUse={close} />
      <TreeNodePicker
        open={dialog === "jumpHost"}
        purpose="jumpHost"
        exclude={web01}
        onClose={close}
        onChoose={close}
      />
      <PreferencesDialog open={dialog === "preferences"} onClose={close} />
    </>
  );
}
