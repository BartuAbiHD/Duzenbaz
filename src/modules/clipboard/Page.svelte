<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { listen } from "@tauri-apps/api/event";
  import { call } from "../../lib/api";
  import { notify } from "../../lib/stores/toast";
  import { acceleratorFromEvent, kindIcon, timeLabel, type ClipItem, type ClipList, type ClipSettings } from "../../lib/clip";
  import ClipThumb from "../../lib/components/ClipThumb.svelte";
  import Icon from "../../lib/components/Icon.svelte";

  let query = $state("");
  let category = $state("");
  let items = $state<ClipItem[]>([]);
  let categories = $state<string[]>([]);
  let settings = $state<ClipSettings | null>(null);
  let shortcutActive = $state(true);
  let showSettings = $state(false);
  let ignoredText = $state("");
  let confirmClear = $state(false);

  // Etiketleme
  let editing = $state<number | null>(null);
  let editCategory = $state("");
  let editTags = $state("");

  let unlisten: (() => void) | undefined;

  async function refresh() {
    try {
      const r = await call<ClipList>("clip_list", { query, category });
      items = r.items;
      categories = r.categories;
    } catch {}
  }

  async function loadSettings() {
    try {
      const r = await call<{ settings: ClipSettings; shortcutActive: boolean }>("clip_get_settings");
      settings = r.settings;
      shortcutActive = r.shortcutActive;
      ignoredText = r.settings.ignoredApps.join("\n");
    } catch {}
  }

  async function saveSettings() {
    if (!settings) return;
    try {
      const next = { ...settings, ignoredApps: ignoredText.split(/[\n,]/).map((s) => s.trim()).filter(Boolean) };
      const r = await call<{ settings: ClipSettings; shortcutActive: boolean }>("clip_set_settings", { settings: next });
      settings = r.settings;
      shortcutActive = r.shortcutActive;
      ignoredText = r.settings.ignoredApps.join("\n");
      notify($_("clipboard.saved"));
    } catch {
      loadSettings();
    }
  }

  function captureShortcut(e: KeyboardEvent) {
    e.preventDefault();
    const acc = acceleratorFromEvent(e);
    if (acc && settings) settings.shortcut = acc;
  }

  let timer: ReturnType<typeof setTimeout>;
  function onInput() {
    clearTimeout(timer);
    timer = setTimeout(refresh, 150);
  }

  async function copy(it: ClipItem) {
    try {
      await call("clip_copy", { id: it.id, paste: false });
      notify($_("clipboard.copied"));
    } catch {}
  }

  async function pin(it: ClipItem) {
    await call("clip_pin", { id: it.id, pinned: !it.pinned });
    refresh();
  }

  async function remove(it: ClipItem) {
    await call("clip_delete", { id: it.id });
    refresh();
  }

  function startEdit(it: ClipItem) {
    editing = it.id;
    editCategory = it.category;
    editTags = it.tags.join(", ");
  }

  async function saveEdit() {
    if (editing === null) return;
    await call("clip_set_meta", { id: editing, category: editCategory, tags: editTags.split(",") });
    editing = null;
    refresh();
  }

  async function clearAll() {
    const n = await call<number>("clip_clear", { keepPinned: true });
    notify($_("clipboard.cleared", { values: { n } }));
    confirmClear = false;
    refresh();
  }

  onMount(async () => {
    loadSettings();
    refresh();
    unlisten = await listen("clipboard-changed", refresh);
  });
  onDestroy(() => unlisten?.());

  const input = "rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800";
  const ghost = "rounded-lg border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700";
  const small = "rounded px-2 py-1 text-xs hover:bg-slate-100 dark:hover:bg-slate-700";
  const card = "rounded-lg border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-800";
</script>

<div class="mb-4 flex items-center justify-between">
  <h1 class="text-2xl font-bold">{$_("clipboard.title")}</h1>
  <div class="flex items-center gap-2">
    {#if settings}
      <button class={ghost} onclick={() => { settings!.paused = !settings!.paused; saveSettings(); }}>
        {settings.paused ? $_("clipboard.resume") : $_("clipboard.pause")}
      </button>
    {/if}
    <button class={ghost} onclick={() => (showSettings = !showSettings)}>{$_("clipboard.settings")}</button>
  </div>
</div>

{#if settings?.paused}
  <p class="mb-3 rounded-lg bg-amber-50 p-3 text-sm text-amber-900 dark:bg-amber-500/10 dark:text-amber-200">{$_("clipboard.pausedNote")}</p>
{/if}
{#if !shortcutActive}
  <p class="mb-3 rounded-lg bg-amber-50 p-3 text-sm text-amber-900 dark:bg-amber-500/10 dark:text-amber-200">{$_("clipboard.shortcutFailed")}</p>
{/if}

{#if showSettings && settings}
  <section class="{card} mb-4 flex flex-col gap-4 p-4 text-sm">
    <div class="grid gap-4 sm:grid-cols-2">
      <label class="block">
        <span class="mb-1 block text-xs font-medium text-slate-500">{$_("clipboard.shortcut")}</span>
        <input class="{input} w-full" readonly value={settings.shortcut} onkeydown={captureShortcut} placeholder={$_("clipboard.shortcutHint")} />
      </label>
      <label class="block">
        <span class="mb-1 block text-xs font-medium text-slate-500">{$_("clipboard.maxItems")}</span>
        <input class="{input} w-full" type="number" min="50" max="5000" bind:value={settings.maxItems} />
      </label>
    </div>
    <label class="flex items-center gap-2"><input type="checkbox" bind:checked={settings.autoPaste} />{$_("clipboard.autoPaste")}</label>
    <label class="block">
      <span class="mb-1 block text-xs font-medium text-slate-500">{$_("clipboard.ignoredApps")}</span>
      <textarea class="{input} h-24 w-full font-mono text-xs" bind:value={ignoredText}></textarea>
      <span class="mt-1 block text-xs text-slate-400">{$_("clipboard.ignoredHint")}</span>
    </label>
    <div><button class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500" onclick={saveSettings}>{$_("clipboard.save")}</button></div>
  </section>
{/if}

<div class="mb-3 flex flex-wrap items-center gap-2">
  <input class="{input} min-w-[14rem] flex-1" placeholder={$_("clipboard.search")} bind:value={query} oninput={onInput} />
  <select class={input} bind:value={category} onchange={refresh}>
    <option value="">{$_("clipboard.allCategories")}</option>
    {#each categories as c}<option value={c}>{c}</option>{/each}
  </select>
  {#if confirmClear}
    <button class="rounded-lg bg-red-600 px-3 py-1.5 text-sm text-white" onclick={clearAll}>{$_("clipboard.clearYes")}</button>
    <button class={ghost} onclick={() => (confirmClear = false)}>{$_("organizer.cancel")}</button>
  {:else}
    <button class={ghost} onclick={() => (confirmClear = true)} disabled={items.length === 0}>{$_("clipboard.clear")}</button>
  {/if}
</div>

{#if items.length === 0}
  <p class="text-slate-500">{$_("clipboard.empty")}</p>
{:else}
  <div class="flex flex-col gap-2">
    {#each items as it (it.id)}
      <div class="{card} p-3">
        <div class="flex items-start gap-3">
          {#if it.kind === "image"}
            <ClipThumb id={it.id} size={64} />
            <div class="flex-1 text-sm text-slate-500">{it.width}×{it.height}</div>
          {:else}
            <Icon name={kindIcon[it.kind]} class="mt-0.5 text-slate-400" />
            <div class="line-clamp-3 min-w-0 flex-1 break-all text-sm">{it.preview}</div>
          {/if}
          <span class="whitespace-nowrap text-xs text-slate-400">{timeLabel(it.updatedMs)}</span>
        </div>
        {#if it.category || it.tags.length}
          <div class="mt-2 flex flex-wrap gap-1.5 text-xs">
            {#if it.category}<span class="rounded bg-indigo-100 px-2 py-0.5 text-indigo-800 dark:bg-indigo-500/20 dark:text-indigo-300">{it.category}</span>{/if}
            {#each it.tags as t}<span class="rounded bg-slate-100 px-2 py-0.5 dark:bg-slate-700">#{t}</span>{/each}
          </div>
        {/if}
        <div class="mt-2 flex gap-1">
          <button class={small} onclick={() => copy(it)}>{$_("clipboard.copy")}</button>
          <button class={small} onclick={() => pin(it)}>{it.pinned ? $_("clipboard.unpin") : $_("clipboard.pin")}</button>
          <button class={small} onclick={() => startEdit(it)}>{$_("clipboard.label")}</button>
          <button class="{small} text-red-600" onclick={() => remove(it)}>{$_("clipboard.delete")}</button>
          {#if it.pinned}<Icon name="pin" size={14} class="ml-auto self-center text-indigo-500" />{/if}
        </div>
        {#if editing === it.id}
          <div class="mt-2 flex flex-wrap items-end gap-2">
            <label class="text-xs"><span class="mb-1 block text-slate-500">{$_("clipboard.category")}</span><input class={input} list="cats" bind:value={editCategory} /></label>
            <label class="text-xs"><span class="mb-1 block text-slate-500">{$_("clipboard.tags")}</span><input class={input} bind:value={editTags} placeholder="iş, fatura" /></label>
            <button class="rounded-lg bg-indigo-600 px-3 py-1.5 text-sm text-white" onclick={saveEdit}>{$_("clipboard.save")}</button>
            <button class={ghost} onclick={() => (editing = null)}>{$_("organizer.cancel")}</button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
  <datalist id="cats">{#each categories as c}<option value={c}></option>{/each}</datalist>
{/if}
