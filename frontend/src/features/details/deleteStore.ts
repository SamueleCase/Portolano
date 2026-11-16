import { create } from "zustand";

import type { DeletePreview } from "../../bindings";

/** The deletion waiting for its confirmation (inventario §5.11). */
export interface DeleteConfirmState {
  /** What `preview_delete` said; none with no confirmation open. */
  preview: DeletePreview | null;
  /** The name of the node, for the title. */
  name: string;
  ask: (preview: DeletePreview, name: string) => void;
  dismiss: () => void;
}

export const useDeleteConfirm = create<DeleteConfirmState>()((set) => ({
  preview: null,
  name: "",
  ask: (preview, name) => set({ preview, name }),
  dismiss: () => set({ preview: null, name: "" }),
}));
