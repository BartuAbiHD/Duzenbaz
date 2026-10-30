<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { call } from "../../lib/api";
  import { notify } from "../../lib/stores/toast";

  type Item = { src: string; oldName: string; newName: string; status: "ok" | "unchanged" | "conflict" | "invalid"; message: string };
  type Batch = { batchId: string; createdAt: string; total: number; remaining: number };

  const dateFormats = [
    ["%Y-%m-%d", "2024-03-15"],
    ["%Y%m%d", "20240315"],
    ["%d-%m-%Y", "15-03-2024"],
    ["%Y-%m-%d_%H-%M-%S", "2024-03-15_14-30-00"],
    ["%Y%m%d_%H%M%S", "20240315_143000"],
  ];

  let paths = $state<string[]>([]);
  let items = $state<Item[]>([]);
  let dragging = $state(false);
  let busy = $state(false);
  let lastBatch = $state<Batch | null>(null);

  let rules = $state({
    sort: "none",
    find: "",
    replace: "",
    caseSensitive: false,
    case: "none",
    date: { enabled: false, source: "exif", format: "%Y-%m-%d", position: "prefix", separator: "_" },
    number: { enabled: false, start: 1, step: 1, digits: 3, position: "suffix", separator: "_" },
    prefix: "",
    suffix: "",
  });

  const changes = $derived(items.filter((i) => i.status === "ok").length);
  const blocked = $derived(items.some((i) => i.status === "conflict" || i.status === "invalid"));

  // Sayısal alanlar boşaltılırsa null gelir; sunucuya göndermeden önce düzeltilir.
  function payload() {
    const r = $state.snapshot(rules);
    r.number.start = Number(r.number.start) || 0;
    r.number.step = Number(r.number.step) || 0;
    r.number.digits = Math.max(0, Math.floor(Number(r.number.digits) || 0));
    return r;
  }

  let timer: ReturnType<typeof setTimeout>;
  let seq = 0;
  $effect(() => {
    const p = [...paths];
    const r = payload();
    clearTimeout(timer);
    if (!p.length) {
      items = [];
      return;
    }
    timer = setTimeout(async () => {
      const mine = ++seq;
      try {
        const res = await call<Item[]>("renamer_preview", { paths: p, rules: r });
        if (mine === seq) items = res;
      } catch {}
    }, 150);
  });

  async function addPaths(list: string[]) {
    try {
      const files = await call<string[]>("renamer_expand", { paths: list });
      const known = new Set(paths.map((p) => p.toLowerCase()));
      paths = [...paths, ...files.filter((f) => !known.has(f.toLowerCase()))];
    } catch {}
  }

  async function pick() {
    const sel = await open({ multiple: true });
    if (Array.isArray(sel)) await addPaths(sel);
  }

  async function loadLast() {
    try {
      const h = await call<Batch[]>("renamer_history");
      lastBatch = h.find((b) => b.remaining > 0) ?? null;
    } catch {}
  }

  async function apply() {
    busy = true;
    try {
      const r = await call<{ renamed: number }>("renamer_apply", { paths: [...paths], rules: payload() });
      notify($_("renamer.renamed", { values: { n: r.renamed } }));
      // Yeni adlar listede kalsın diye yollar güncellenir.
      const renamedMap = new Map(items.filter((i) => i.status === "ok").map((i) => [i.src, i.src.replace(/[^\\/]+$/, i.newName)]));
      paths = paths.map((p) => renamedMap.get(p) ?? p);
      resetRules();
      await loadLast();
    } catch {
    } finally {
      busy = false;
    }
  }

  async function undo() {
    if (!lastBatch) return;
    busy = true;
    try {
      const r = await call<{ restored: number }>("renamer_undo", { batchId: lastBatch.batchId });
      notify($_("renamer.undone", { values: { n: r.restored } }));
      paths = [];
      await loadLast();
    } catch {
    } finally {
      busy = false;
    }
  }

  function resetRules() {
    rules.find = rules.replace = rules.prefix = rules.suffix = "";
    rules.case = "none";
    rules.date.enabled = false;
    rules.number.enabled = false;
  }

  let unlisten: (() => void) | undefined;
  onMount(async () => {
    loadLast();
    unlisten = await getCurrentWebview().onDragDropEvent((e) => {
      const t = e.payload.type;
      if (t === "enter" || t === "over") dragging = true;
      else if (t === "drop") {
        dragging = false;
        addPaths(e.payload.paths);
      } else dragging = false;
    });
  });
  onDestroy(() => unlisten?.());

  const input = "w-full rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800";
  const label = "mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400";
  const box = "rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-700 dark:bg-slate-800";
  const btn = "rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50";
  const ghost = "rounded-lg border border-slate-300 px-4 py-2 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700";
  const positions = [
    ["prefix", "renamer.posPrefix"],
    ["suffix", "renamer.posSuffix"],
    ["replace", "renamer.posReplace"],
  ];
</script>

<h1 class="mb-4 text-2xl font-bold">{$_("renamer.title")}</h1>

<div
  class="mb-4 flex items-center gap-4 rounded-xl border-2 border-dashed px-5 py-5 text-sm transition-colors {dragging
    ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-500/10'
    : 'border-slate-300 dark:border-slate-600'}"
>
  <span class="flex-1 text-slate-500 dark:text-slate-400">{dragging ? $_("renamer.dropActive") : $_("renamer.drop")}</span>
  <button class={ghost} onclick={pick}>{$_("renamer.add")}</button>
  <button class={ghost} onclick={() => (paths = [])} disabled={!paths.length}>{$_("renamer.clear")}</button>
</div>

<div class="mb-4 grid gap-4 lg:grid-cols-2">
  <section class={box}>
    <div class="mb-2 text-sm font-semibold">{$_("renamer.findReplace")}</div>
    <div class="grid grid-cols-2 gap-3">
      <div><span class={label}>{$_("renamer.find")}</span><input class={input} bind:value={rules.find} /></div>
      <div><span class={label}>{$_("renamer.replaceWith")}</span><input class={input} bind:value={rules.replace} /></div>
    </div>
    <label class="mt-2 flex items-center gap-2 text-xs"><input type="checkbox" bind:checked={rules.caseSensitive} />{$_("renamer.caseSensitive")}</label>

    <div class="mt-4 grid grid-cols-2 gap-3">
      <div><span class={label}>{$_("renamer.prefix")}</span><input class={input} bind:value={rules.prefix} /></div>
      <div><span class={label}>{$_("renamer.suffix")}</span><input class={input} bind:value={rules.suffix} /></div>
      <div>
        <span class={label}>{$_("renamer.letterCase")}</span>
        <select class={input} bind:value={rules.case}>
          <option value="none">{$_("renamer.caseNone")}</option>
          <option value="lower">{$_("renamer.caseLower")}</option>
          <option value="upper">{$_("renamer.caseUpper")}</option>
          <option value="title">{$_("renamer.caseTitle")}</option>
          <option value="sentence">{$_("renamer.caseSentence")}</option>
        </select>
      </div>
      <div>
        <span class={label}>{$_("renamer.sort")}</span>
        <select class={input} bind:value={rules.sort}>
          <option value="none">{$_("renamer.sortNone")}</option>
          <option value="name">{$_("renamer.sortName")}</option>
          <option value="modified">{$_("renamer.sortModified")}</option>
        </select>
      </div>
    </div>
  </section>

  <div class="flex flex-col gap-4">
    <section class={box}>
      <label class="flex items-center gap-2 text-sm font-semibold"><input type="checkbox" bind:checked={rules.date.enabled} />{$_("renamer.date")}</label>
      {#if rules.date.enabled}
        <div class="mt-3 grid grid-cols-2 gap-3">
          <div class="col-span-2">
            <span class={label}>{$_("renamer.dateSource")}</span>
            <select class={input} bind:value={rules.date.source}>
              <option value="exif">{$_("renamer.srcExif")}</option>
              <option value="modified">{$_("renamer.srcModified")}</option>
              <option value="created">{$_("renamer.srcCreated")}</option>
            </select>
          </div>
          <div>
            <span class={label}>{$_("renamer.dateFormat")}</span>
            <select class={input} bind:value={rules.date.format}>
              {#each dateFormats as [v, l]}<option value={v}>{l}</option>{/each}
            </select>
          </div>
          <div>
            <span class={label}>{$_("renamer.position")}</span>
            <select class={input} bind:value={rules.date.position}>
              {#each positions as [v, k]}<option value={v}>{$_(k)}</option>{/each}
            </select>
          </div>
          <div><span class={label}>{$_("renamer.separator")}</span><input class={input} bind:value={rules.date.separator} /></div>
        </div>
      {/if}
    </section>

    <section class={box}>
      <label class="flex items-center gap-2 text-sm font-semibold"><input type="checkbox" bind:checked={rules.number.enabled} />{$_("renamer.number")}</label>
      {#if rules.number.enabled}
        <div class="mt-3 grid grid-cols-3 gap-3">
          <div><span class={label}>{$_("renamer.start")}</span><input type="number" class={input} bind:value={rules.number.start} /></div>
          <div><span class={label}>{$_("renamer.step")}</span><input type="number" class={input} bind:value={rules.number.step} /></div>
          <div><span class={label}>{$_("renamer.digits")}</span><input type="number" min="0" max="12" class={input} bind:value={rules.number.digits} /></div>
          <div class="col-span-2">
            <span class={label}>{$_("renamer.position")}</span>
            <select class={input} bind:value={rules.number.position}>
              {#each positions as [v, k]}<option value={v}>{$_(k)}</option>{/each}
            </select>
          </div>
          <div><span class={label}>{$_("renamer.separator")}</span><input class={input} bind:value={rules.number.separator} /></div>
        </div>
      {/if}
    </section>
  </div>
</div>

<div class="mb-2 flex items-center justify-between">
  <div class="text-sm font-semibold">{$_("renamer.preview")}</div>
  {#if lastBatch}
    <button class="text-xs text-indigo-600 underline dark:text-indigo-400" onclick={undo} disabled={busy}>
      {$_("renamer.undoLast", { values: { n: lastBatch.remaining } })}
    </button>
  {/if}
</div>

{#if items.length === 0}
  <p class="mb-4 text-sm text-slate-500">{$_("renamer.empty")}</p>
{:else}
  <div class="{box} mb-4 max-h-[40vh] overflow-y-auto !p-0">
    <table class="w-full text-sm">
      <thead class="sticky top-0 bg-slate-100 text-left text-xs text-slate-500 dark:bg-slate-700 dark:text-slate-300">
        <tr><th class="px-3 py-2">{$_("renamer.colOld")}</th><th></th><th class="py-2">{$_("renamer.colNew")}</th></tr>
      </thead>
      <tbody>
        {#each items as it (it.src)}
          <tr class="border-t border-slate-100 dark:border-slate-700/60">
            <td class="max-w-[16rem] truncate px-3 py-1.5" title={it.src}>{it.oldName}</td>
            <td class="px-2 text-slate-400">→</td>
            <td class="py-1.5 pr-3">
              {#if it.status === "unchanged"}
                <span class="text-slate-400">{$_("renamer.statusUnchanged")}</span>
              {:else}
                <span class={it.status === "ok" ? "text-emerald-700 dark:text-emerald-400" : "text-red-600 dark:text-red-400"}>{it.status === "invalid" ? "—" : it.newName}</span>
              {/if}
              {#if it.message}<div class="text-xs {it.status === 'ok' ? 'text-amber-600' : 'text-red-500'}">{it.message}</div>{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="flex items-center gap-4">
    <button class={btn} onclick={apply} disabled={busy || blocked || changes === 0}>{$_("renamer.apply", { values: { n: changes } })}</button>
    {#if blocked}<span class="text-sm text-red-500">{$_("renamer.blocked")}</span>{/if}
  </div>
{/if}
