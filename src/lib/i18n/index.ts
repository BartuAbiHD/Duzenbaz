import { addMessages, init, locale } from "svelte-i18n";
import tr from "./tr.json";
import en from "./en.json";

export const LOCALES = [
  { code: "tr", label: "Türkçe" },
  { code: "en", label: "English" },
];

function saved(): string {
  try {
    return localStorage.getItem("locale") ?? "tr";
  } catch {
    return "tr";
  }
}

addMessages("tr", tr);
addMessages("en", en);
init({ fallbackLocale: "tr", initialLocale: saved() });

export function setLocale(code: string) {
  locale.set(code);
  try {
    localStorage.setItem("locale", code);
  } catch {}
}
