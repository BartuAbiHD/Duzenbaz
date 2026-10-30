import { writable } from "svelte/store";

export interface Toast {
  id: number;
  kind: "error" | "info";
  text: string;
}

export const toasts = writable<Toast[]>([]);
let next = 1;

export function notify(text: string, kind: Toast["kind"] = "info", ms = 6000) {
  const id = next++;
  toasts.update((l) => [...l, { id, kind, text }]);
  setTimeout(() => toasts.update((l) => l.filter((t) => t.id !== id)), ms);
}
