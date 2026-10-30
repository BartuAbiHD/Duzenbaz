import { invoke } from "@tauri-apps/api/core";
import { notify } from "./stores/toast";

/** Rust komutunu çağırır; hata gelirse Türkçe mesajı kullanıcıya gösterir ve tekrar fırlatır. */
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    const msg = typeof e === "string" ? e : "Beklenmeyen bir hata oluştu.";
    notify(msg, "error");
    throw e;
  }
}
