import { useQueryClient } from "@tanstack/react-query";

import { revealNode } from "../features/tree/useGoToNode";
import { treeQuery } from "../ipc/queries";
import {
  answerLogin,
  answerVault,
  closeRequest,
  type OpenRequest,
  useRequests,
} from "../state/requests";
import { useFeatures } from "./features";

/**
 * Mounts the function of every open request (`features.ts`), the last one on
 * top: a function that asks for another (the editor that needs the vault)
 * stays mounted below with its state. The editor and the import end on a
 * node, which the window then selects once the tree has it.
 */
export function FeatureHost() {
  const stack = useRequests((state) => state.stack);
  return stack.map((open) => <OpenFeature key={open.id} open={open} />);
}

function OpenFeature({ open }: { open: OpenRequest }) {
  const queryClient = useQueryClient();
  const features = useFeatures();
  const { id, request } = open;
  const close = () => closeRequest(id);
  const goToNode = (node: string) =>
    void queryClient
      .fetchQuery({ ...treeQuery, staleTime: 0 })
      .catch(() => undefined)
      .then(() => revealNode(queryClient, node));

  switch (request.kind) {
    case "editor": {
      const Editor = features.editor;
      return Editor ? <Editor target={request.target} onClose={close} onSaved={goToNode} /> : null;
    }
    case "password": {
      const Password = features.password;
      return Password ? (
        <Password node={request.node} field={request.field} onClose={close} />
      ) : null;
    }
    case "unlockVault": {
      const Unlock = features.unlockVault;
      return Unlock ? <Unlock onDone={answerVault} /> : null;
    }
    case "createFileVault": {
      const Create = features.createFileVault;
      return Create ? <Create onDone={answerVault} /> : null;
    }
    case "login": {
      const Login = features.login;
      return Login ? <Login request={request.request} onDone={answerLogin} /> : null;
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
