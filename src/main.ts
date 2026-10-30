import { mount } from "svelte";
import "./app.css";
import "./lib/i18n";
import App from "./App.svelte";
import Popup from "./modules/clipboard/Popup.svelte";
import Break from "./modules/breaks/Break.svelte";
import "./lib/stores/theme";
import { lockdown } from "./lib/lockdown";

lockdown();

// Aynı arayüz paketi üç pencerede kullanılır: ana pencere, kısayolla açılan pano penceresi ve mola penceresi.
const view = new URLSearchParams(location.search).get("view");
const Root = view === "popup" ? Popup : view === "break" ? Break : App;

export default mount(Root, { target: document.getElementById("app")! });
