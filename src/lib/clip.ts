import { get } from "svelte/store";
import { locale } from "svelte-i18n";
import { timeFormat } from "./stores/timeFormat";

export type ClipItem = {
  id: number;
  kind: "text" | "link" | "image";
  preview: string;
  category: string;
  tags: string[];
  pinned: boolean;
  updatedMs: number;
  size: number;
  width: number | null;
  height: number | null;
};

export type ClipList = { items: ClipItem[]; categories: string[] };

export type ClipSettings = {
  paused: boolean;
  maxItems: number;
  ignoredApps: string[];
  shortcut: string;
  autoPaste: boolean;
};

export const kindIcon = { text: "file-text", link: "link", image: "image" } as const;

export function timeLabel(ms: number): string {
  // Tarih, işletim sisteminin değil uygulamada seçili dilin biçiminde gösterilir.
  return new Date(ms).toLocaleString(get(locale) ?? "tr", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    hour12: get(timeFormat) === "12",
  });
}

/** Tuş basışını Tauri kısayol biçimine çevirir (örn. "Ctrl+Shift+V"). Değiştirici tuş yoksa null. */
export function acceleratorFromEvent(e: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean) as string[];
  if (!mods.length) return null;
  const c = e.code;
  const key = c.startsWith("Key") ? c.slice(3) : c.startsWith("Digit") ? c.slice(5) : c;
  return [...mods, key].join("+");
}
