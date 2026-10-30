import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { _ } from "svelte-i18n";
import { notify } from "./toast";

/** Güncelleme kontrolü varsayılan olarak kapalıdır; açılmadıkça uygulama internete hiç bağlanmaz. */
function read(): boolean {
  try {
    return localStorage.getItem("updateCheck") === "on";
  } catch {
    return false;
  }
}

export const autoCheck = writable<boolean>(read());

autoCheck.subscribe((v) => {
  try {
    localStorage.setItem("updateCheck", v ? "on" : "off");
  } catch {}
});

export interface UpdateState {
  status: "idle" | "checking" | "none" | "available" | "downloading" | "ready" | "error";
  version?: string;
  notes?: string;
  percent?: number;
}

export const updateState = writable<UpdateState>({ status: "idle" });

let pending: Update | null = null;

/** `silent`: açılıştaki otomatik kontrol; yalnızca yeni sürüm varsa bildirir, hata ve "güncelsiniz" göstermez. */
export async function checkForUpdate(silent = false) {
  updateState.set({ status: "checking" });
  try {
    const update = await check();
    if (!update) {
      pending = null;
      updateState.set({ status: "none" });
      return;
    }
    pending = update;
    updateState.set({ status: "available", version: update.version, notes: update.body ?? undefined });
    if (silent) notify(get(_)("settings.updatesToast", { values: { version: update.version } }));
  } catch {
    updateState.set({ status: silent ? "idle" : "error" });
  }
}

export async function installUpdate() {
  if (!pending) return;
  const version = pending.version;
  let total = 0;
  let done = 0;
  updateState.set({ status: "downloading", version, percent: 0 });
  try {
    await pending.downloadAndInstall((e) => {
      if (e.event === "Started") total = e.data.contentLength ?? 0;
      if (e.event === "Progress") {
        done += e.data.chunkLength;
        updateState.set({ status: "downloading", version, percent: total ? Math.round((done / total) * 100) : 0 });
      }
    });
    updateState.set({ status: "ready", version });
  } catch {
    updateState.set({ status: "error", version });
  }
}

export const restartApp = () => invoke("app_restart");
