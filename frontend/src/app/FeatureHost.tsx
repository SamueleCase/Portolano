import { useQueryClient } from "@tanstack/react-query";

import { revealNode } from "../features/tree/useGoToNode";
import { treeQuery } from "../ipc/queries";
import { answerLogin, useRequests } from "../state/requests";
import { useFeatures } from "./features";

/**
 * Mounts the function of the request open now (`features.ts`), one at a time.
 * The editor and the import end on a node, which the window then selects
 * once the tree has it.
 */
export function FeatureHost() {
  const queryClient = useQueryClient();
  const features = useFeatures();
  const active = useRequests((state) => state.active);
  const close = useRequests((state) => state.close);
  const goToNode = (id: string) =>
    void queryClient
      .fetchQuery({ ...treeQuery, staleTime: 0 })
      .then(() => revealNode(queryClient, id))
      .catch(() => revealNode(queryClient, id));

  if (active === null) return null;
  switch (active.kind) {
    case "editor": {
      const Editor = features.editor;
      return Editor ? <Editor target={active.target} onClose={close} onSaved={goToNode} /> : null;
    }
    case "password": {
      const Password = features.password;
      return Password ? <Password node={active.node} field={active.field} onClose={close} /> : null;
    }
    case "unlockVault": {
      const Unlock = features.unlockVault;
      return Unlock ? <Unlock onClose={close} /> : null;
    }
    case "createFileVault": {
      const Create = features.createFileVault;
      return Create ? <Create onClose={close} /> : null;
    }
    case "login": {
      const Login = features.login;
      return Login ? <Login request={active.request} onDone={answerLogin} /> : null;
    }
    case "import": {
      const Import = features.import;
      return Import ? <Import onClose={close} onGoToNode={goToNode} /> : null;
    }
    case "preferences": {
      const Preferences = features.preferences;
      return Preferences ? <Preferences onClose={close} /> : null;
    }
  }
}
