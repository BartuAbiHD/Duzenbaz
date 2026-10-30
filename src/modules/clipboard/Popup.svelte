<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { _ } from "svelte-i18n";
  import { listen } from "@tauri-apps/api/event";
  import { call } from "../../lib/api";
  import { kindIcon, type ClipItem, type ClipList } from "../../lib/clip";
  import ClipThumb from "../../lib/components/ClipThumb.svelte";
  import Icon from "../../lib/components/Icon.svelte";

  let query = $state("");
  let items = $state<ClipItem[]>([]);
  let selected = $state(0);
  let autoPaste = $state(true);
  let searchEl: HTMLInputElement;
  let listEl: HTMLDivElement;
  const unlisten: Array<() => void> = [];

  async function refresh() {
    try {
      const r = await call<ClipList>("clip_list", { query, category: "" });
      items = r.items;
      selected = Math.min(selected, Math.max(0, items.length - 1));
    } catch {}
  }

  async function loadSettings() {
    try {
      autoPaste = (await call<{ settings: { autoPaste: boolean } }>("clip_get_settings")).settings.autoPaste;
    } catch {}
  }

  async function pick(i: number) {
    const it = items[i];
    if (!it) return;
    try {
      await call("clip_copy", { id: it.id, paste: autoPaste });
    } catch {}
  }

  async function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") await call("clip_hide_popup");
    else if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, items.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(selected);
    } else return;
    await tick();
    listEl?.querySelector(`[data-i="${selected}"]`)?.scrollIntoView({ block: "nearest" });
  }

  let timer: ReturnType<typeof setTimeout>;
  function onInput() {
    selected = 0;
    clearTimeout(timer);
    timer = setTimeout(refresh, 120);
  }

  onMount(async () => {
    refresh();
    loadSettings();
    unlisten.push(
      await listen("popup-shown", async () => {
        query = "";
        selected = 0;
        loadSettings();
        await refresh();
        searchEl?.focus();
      }),
      await listen("clipboard-changed", refresh),
    );
    searchEl?.focus();
  });
  onDestroy(() => unlisten.forEach((u) => u()));
</script>

<svelte:window onkeydown={onKey} />

<div class="flex h-full flex-col overflow-hidden border border-slate-300 bg-white dark:border-slate-600 dark:bg-slate-900">
  <input
    bind:this={searchEl}
    bind:value={query}
    oninput={onInput}
    placeholder={$_("clipboard.search")}
    class="m-3 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm outline-none focus:border-indigo-500 dark:border-slate-600 dark:bg-slate-800"
  />
  <div bind:this={listEl} class="flex-1 overflow-y-auto px-2 pb-2">
    {#if items.length === 0}
      <p class="p-4 text-center text-sm text-slate-500">{$_("clipboard.empty")}</p>
    {/if}
    {#each items as it, i (it.id)}
      <button
        data-i={i}
        class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm {i === selected ? 'bg-indigo-50 dark:bg-indigo-500/20' : ''}"
        onclick={() => pick(i)}
        onmousemove={() => (selected = i)}
      >
        {#if it.kind === "image"}
          <ClipThumb id={it.id} size={40} />
          <span class="text-xs text-slate-500">{it.width}×{it.height}</span>
        {:else}
          <Icon name={kindIcon[it.kind]} class="text-slate-400" />
          <span class="line-clamp-2 min-w-0 flex-1 break-all">{it.preview}</span>
        {/if}
        {#if it.pinned}<Icon name="pin" size={14} class="text-indigo-500" />{/if}
      </button>
    {/each}
  </div>
  <div class="border-t border-slate-200 px-3 py-1.5 text-xs text-slate-400 dark:border-slate-700">{$_("clipboard.popupHint")}</div>
</div>
