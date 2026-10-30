<script lang="ts">
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import Organizer from "./modules/organizer/Page.svelte";
  import Renamer from "./modules/renamer/Page.svelte";
  import Clipboard from "./modules/clipboard/Page.svelte";
  import Breaks from "./modules/breaks/Page.svelte";
  import Settings from "./modules/settings/Page.svelte";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import "./lib/stores/theme";
  import { autoCheck, checkForUpdate } from "./lib/stores/updates";

  const items = [
    { id: "organizer", icon: "folders" },
    { id: "renamer", icon: "pencil-line" },
    { id: "clipboard", icon: "clipboard-list" },
    { id: "breaks", icon: "eye" },
    { id: "settings", icon: "settings" },
  ];
  const pages = { organizer: Organizer, renamer: Renamer, clipboard: Clipboard, breaks: Breaks, settings: Settings };

  let current = $state<keyof typeof pages>("organizer");
  const Page = $derived(pages[current]);

  // Yalnızca kullanıcı açtıysa ve yalnızca ana pencerede: açılıştan biraz sonra sessizce sürüm sorar.
  onMount(() => {
    if (!get(autoCheck)) return;
    const t = setTimeout(() => checkForUpdate(true), 8000);
    return () => clearTimeout(t);
  });
</script>

<div class="flex h-full">
  <Sidebar {items} {current} onselect={(id) => (current = id as keyof typeof pages)} />
  <main class="flex-1 overflow-y-auto p-8"><Page /></main>
</div>
<Toasts />
