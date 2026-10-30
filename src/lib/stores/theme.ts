import { writable, get } from "svelte/store";

export type Theme = "system" | "light" | "dark";

function read(): Theme {
  try {
    const v = localStorage.getItem("theme");
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {}
  return "system";
}

export const theme = writable<Theme>(read());

const media = window.matchMedia("(prefers-color-scheme: dark)");

function apply(t: Theme) {
  const dark = t === "dark" || (t === "system" && media.matches);
  document.documentElement.classList.toggle("dark", dark);
}

theme.subscribe((t) => {
  apply(t);
  try {
    localStorage.setItem("theme", t);
  } catch {}
});
media.addEventListener("change", () => apply(get(theme)));
