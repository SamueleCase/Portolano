import { QueryClientProvider, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { FeatureHost } from "../../app/FeatureHost";
import { featureRegistry } from "../../app/featureRegistry";
import { FeatureRegistryContext } from "../../app/features";
import { Button } from "../../components/Button";
import type { CredentialChoice_Deserialize } from "../../bindings";
import { CredentialChooser } from "../../features/dialogs/CredentialChooser";
import { IconChooser } from "../../features/dialogs/IconChooser";
import { TreeNodePicker } from "../../features/dialogs/TreeNodePicker";
import { indexTree } from "../../features/dialogs/treeIndex";
import { type EditorRequest, NodeEditor } from "../../features/editor/NodeEditor";
import { interpolate } from "../../i18n/format";
import { editorGallery } from "../../i18n/editor";
import { gallery } from "../../i18n/gallery";
import { createQueryClient } from "../../ipc/queryClient";
import { loginRequestQuery, treeQuery } from "../../ipc/queries";
import { type HostRequest, openRequest, requestLogin } from "../../state/requests";
import styles from "../Gallery.module.css";
import { GallerySection, Specimen } from "../GallerySection";

type LocalDialog = "credential" | "icon" | "jumpHost";

// The section has its own cache: in the e2e build the fake backend answers it.
const queryClient = createQueryClient();

/**
 * The editor, the dialogs and the preferences of UI-WP5 on the fake backend
 * (`npm run e2e`), with the sample document: nodes are found by name. The
 * dialogs of the hook points open as in the window, through its requests
 * and its host (`app/FeatureHost`).
 */
export function EditorSection() {
  return (
    <QueryClientProvider client={queryClient}>
      <FeatureRegistryContext value={featureRegistry}>
        <GallerySection id="editor" title={gallery.section.editor}>
          <EditorSpecimens />
        </GallerySection>
        <FeatureHost />
      </FeatureRegistryContext>
    </QueryClientProvider>
  );
}

function EditorSpecimens() {
  const t = editorGallery.open;
  const client = useQueryClient();
  const { data: tree } = useQuery(treeQuery);
  const entries = tree ? indexTree(tree) : [];
  const idOf = (name: string) => entries.find((entry) => entry.node.name === name)?.node.id;
  const [request, setRequest] = useState<EditorRequest | null>(null);
  const [dialog, setDialog] = useState<LocalDialog>();
  const [credential, setCredential] = useState<CredentialChoice_Deserialize>({ mode: "inherit" });
  const [result, setResult] = useState("");
  const close = () => setDialog(undefined);

  const edit = (name: string) => {
    const id = idOf(name);
    if (id) setRequest({ mode: "edit", id });
  };
  const add = (kind: "ssh" | "rdp" | "folder" | "credential", parent: string | null) =>
    setRequest({ mode: "add", kind, parent: parent === null ? null : (idOf(parent) ?? null) });
  const ask = (hostRequest: HostRequest) => void openRequest(hostRequest);
  const askLogin = async () => {
    const id = idOf("kiosk");
    if (!id) return;
    const asked = await client.fetchQuery({ ...loginRequestQuery(id), staleTime: 0 });
    if (asked === null) {
      setResult(editorGallery.noLogin);
      return;
    }
    const login = await requestLogin(asked);
    setResult(
      login === null
        ? editorGallery.cancelled
        : interpolate(editorGallery.login, { user: login.username ?? "" }),
    );
  };
  const admin = idOf("Admin ACME");

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
            <Button
              onClick={() => {
                if (admin) {
                  ask({
                    kind: "password",
                    node: { id: admin, path: "/Clienti/Admin ACME" },
                    field: { kind: "password" },
                  });
                }
              }}
            >
              {t.password}
            </Button>
            <Button onClick={() => ask({ kind: "unlockVault" })}>{t.unlock}</Button>
            <Button onClick={() => ask({ kind: "createFileVault" })}>{t.create}</Button>
            <Button onClick={() => void askLogin()}>{t.login}</Button>
            <Button onClick={() => setDialog("credential")}>{t.credential}</Button>
            <Button onClick={() => setDialog("icon")}>{t.icon}</Button>
            <Button onClick={() => setDialog("jumpHost")}>{t.jumpHost}</Button>
            <Button onClick={() => ask({ kind: "preferences" })}>{t.preferences}</Button>
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
        exclude={idOf("web01")}
        onClose={close}
        onChoose={close}
      />
    </>
  );
}
