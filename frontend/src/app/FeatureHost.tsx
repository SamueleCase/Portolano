import { useQueryClient } from "@tanstack/react-query";
import { type ReactElement, useEffect } from "react";

import { revealNode } from "../features/tree/useGoToNode";
import { treeQuery } from "../ipc/queries";
import { answerRequest, closeRequest, type OpenRequest, useRequests } from "../state/requests";
import { type FeatureRegistry, useFeatures } from "./features";

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

/** The component of request `open`, or `null` when its function is not mounted. */
function featureOf(
  { id, request }: OpenRequest,
  features: FeatureRegistry,
  goToNode: (node: string) => void,
): ReactElement | null {
  const close = () => closeRequest(id);
  // Each answer goes to its own request, wherever it is in the stack.
  const answer = (value: boolean) => answerRequest(id, value);
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
      return Unlock ? <Unlock onDone={answer} /> : null;
    }
    case "createFileVault": {
      const Create = features.createFileVault;
      return Create ? <Create onDone={answer} /> : null;
    }
    case "login": {
      const Login = features.login;
      return Login ? (
        <Login request={request.request} onDone={(login) => answerRequest(id, login)} />
      ) : null;
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

function OpenFeature({ open }: { open: OpenRequest }) {
  const queryClient = useQueryClient();
  const features = useFeatures();
  const goToNode = (node: string) =>
    void queryClient
      .fetchQuery({ ...treeQuery, staleTime: 0 })
      .catch(() => undefined)
      .then(() => revealNode(queryClient, node));
  const element = featureOf(open, features, goToNode);
  const missing = element === null;

  // A request whose function is not mounted closes at once: who waits for it
  // gets "no" (`null`, `false`) and the window is not left waiting.
  useEffect(() => {
    if (missing) closeRequest(open.id);
  }, [missing, open.id]);

  return element;
}
