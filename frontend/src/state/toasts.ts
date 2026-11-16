import { create } from "zustand";

import type { ToastMessage } from "../components/Toast";

/** A toast to show; the store gives it its id. */
export type NewToast = Omit<ToastMessage, "id">;

/**
 * The toasts of the window (ambiguity 17). The viewport shows the newest three
 * and each toast leaves by itself after its time; errors stay until closed.
 * The texts never carry a secret: only names, paths and messages of the
 * backend.
 */
export interface ToastsState {
  toasts: ToastMessage[];
  /** Shows `toast`; returns its id. */
  show: (toast: NewToast) => string;
  dismiss: (id: string) => void;
}

let nextId = 0;

export const useToasts = create<ToastsState>()((set) => ({
  toasts: [],
  show: (toast) => {
    nextId += 1;
    const id = `toast-${nextId}`;
    set(({ toasts }) => ({ toasts: [...toasts, { ...toast, id }] }));
    return id;
  },
  dismiss: (id) => set(({ toasts }) => ({ toasts: toasts.filter((toast) => toast.id !== id) })),
}));

/** Shows `toast` from outside React (the commands of the window). */
export function showToast(toast: NewToast): string {
  return useToasts.getState().show(toast);
}
