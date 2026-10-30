import { writable } from "svelte/store";

/** "24" = 14:30, "12" = 02:30 PM. Varsayılan 24 saat (Türkiye). */
export type TimeFormat = "24" | "12";

function read(): TimeFormat {
  try {
    return localStorage.getItem("timeFormat") === "12" ? "12" : "24";
  } catch {
    return "24";
  }
}

export const timeFormat = writable<TimeFormat>(read());

timeFormat.subscribe((v) => {
  try {
    localStorage.setItem("timeFormat", v);
  } catch {}
});
