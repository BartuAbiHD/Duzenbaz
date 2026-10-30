<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { open } from "@tauri-apps/plugin-dialog";
  import { call } from "../../lib/api";
  import { notify } from "../../lib/stores/toast";

  type Move = { src: string; dst: string; category: string; size: number };
  type DupFile = { path: string; modified: string };
  type Group = { hash: string; size: number; files: DupFile[] };
  type Failure = { path: string; message: string };
  type Batch = { batchId: string; createdAt: string; total: number; remaining: number };

  let root = $state("");
  let tab = $state<"organize" | "dupes" | "history">("organize");
  let busy = $state(false);

  // Düzenle
  let byDate = $state(false);
  let plan = $state<Move[]>([]);
  let picked = $state<Set<string>>(new Set());
  let planned = $state(false);
  let failures = $state<Failure[]>([]);

  // Kopyalar
  let recursive = $state(false);
  let groups = $state<Group[]>([]);
  let scannedDupes = $state(false);
  let removeSet = $state<Set<string>>(new Set()); // silinmek üzere işaretlenen yollar
  let confirming = $state(false);

  // Geçmiş
  let history = $state<Batch[]>([]);

  const fmtSize = (b: number) => {
    const u = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    while (b >= 1024 && i < u.length - 1) {
      b /= 1024;
      i++;
    }
    return `${b.toFixed(i ? 1 : 0)} ${u[i]}`;
  };
  const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;
  const reclaimable = $derived(groups.reduce((s, g) => s + g.size * (g.files.length - 1), 0));

  onMount(async () => {
    try {
      root = await call<string>("default_downloads");
    } catch {}
    loadHistory();
  });

  async function chooseFolder() {
    const dir = await open({ directory: true, defaultPath: root || undefined });
    if (typeof dir === "string") {
      root = dir;
      plan = [];
      planned = false;
      groups = [];
      scannedDupes = false;
    }
  }

  async function preview() {
    busy = true;
    failures = [];
    try {
      plan = await call<Move[]>("organizer_plan", { root, options: { byDate } });
      picked = new Set(plan.map((m) => m.src));
      planned = true;
    } catch {
    } finally {
      busy = false;
    }
  }

  async function apply() {
    busy = true;
    try {
      const r = await call<{ batchId: string; moved: number; failed: Failure[] }>("organizer_apply", {
        root,
        options: { byDate },
        selected: [...picked],
      });
      notify($_("organizer.moved", { values: { n: r.moved } }));
      failures = r.failed;
      plan = [];
      planned = false;
      await loadHistory();
    } catch {
    } finally {
      busy = false;
    }
  }

  function toggle(set: Set<string>, key: string): Set<string> {
    const n = new Set(set);
    n.has(key) ? n.delete(key) : n.add(key);
    return n;
  }

  async function findDupes() {
    busy = true;
    try {
      groups = await call<Group[]>("organizer_find_duplicates", { root, recursive });
      scannedDupes = true;
      confirming = false;
      // Her grupta en eski dosya saklanır, diğerleri varsayılan olarak işaretlenir.
      removeSet = new Set(groups.flatMap((g) => g.files.slice(1).map((f) => f.path)));
    } catch {
    } finally {
      busy = false;
    }
  }

  function setKeeper(g: Group, path: string) {
    const n = new Set(removeSet);
    for (const f of g.files) f.path === path ? n.delete(f.path) : n.add(f.path);
    removeSet = n;
  }

  function toggleRemove(g: Group, path: string) {
    const next = toggle(removeSet, path);
    // Gruptaki tüm kopyaları işaretlemeye izin verme: en az biri kalmalı.
    if (g.files.every((f) => next.has(f.path))) return;
    removeSet = next;
  }

  async function trashSelected() {
    busy = true;
    try {
      const requests = groups
        .map((g) => ({
          keep: g.files.find((f) => !removeSet.has(f.path))?.path ?? "",
          remove: g.files.filter((f) => removeSet.has(f.path)).map((f) => f.path),
        }))
        .filter((r) => r.keep && r.remove.length);
      const r = await call<{ trashed: number; skipped: number }>("organizer_trash_duplicates", { requests });
      notify($_("organizer.trashed", { values: { n: r.trashed } }));
      confirming = false;
      await findDupes();
    } catch {
    } finally {
      busy = false;
    }
  }

  async function loadHistory() {
    try {
      history = await call<Batch[]>("organizer_history");
    } catch {}
  }

  async function undo(id: string) {
    busy = true;
    try {
      const r = await call<{ restored: number; failed: Failure[] }>("organizer_undo", { batchId: id });
      notify($_("organizer.undone", { values: { n: r.restored } }));
      failures = r.failed;
      await loadHistory();
    } catch {
    } finally {
      busy = false;
    }
  }

  const tabs = [
    ["organize", "organizer.tabOrganize"],
    ["dupes", "organizer.tabDupes"],
    ["history", "organizer.tabHistory"],
  ] as const;
  const btn = "rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50";
  const ghost = "rounded-lg border border-slate-300 px-4 py-2 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700";
  const card = "rounded-lg border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-800";
</script>

<h1 class="mb-4 text-2xl font-bold">{$_("organizer.title")}</h1>

<div class="mb-4 flex items-center gap-3">
  <div class="min-w-0 flex-1 truncate rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800" title={root}>
    {root || "—"}
  </div>
  <button class={ghost} onclick={chooseFolder} disabled={busy}>{$_("organizer.choose")}</button>
</div>

<div class="mb-5 flex gap-1 border-b border-slate-200 dark:border-slate-700">
  {#each tabs as [id, key] (id)}
    <button
      class="-mb-px border-b-2 px-4 py-2 text-sm {tab === id ? 'border-indigo-600 font-semibold text-indigo-600 dark:text-indigo-400' : 'border-transparent text-slate-500'}"
      onclick={() => {
        tab = id;
        if (id === "history") loadHistory();
      }}>{$_(key)}</button
    >
  {/each}
</div>

{#if failures.length}
  <div class="mb-4 rounded-lg bg-amber-50 p-3 text-sm text-amber-900 dark:bg-amber-500/10 dark:text-amber-200">
    <div class="mb-1 font-semibold">{$_("organizer.failedSome", { values: { n: failures.length } })}</div>
    <ul class="list-inside list-disc">
      {#each failures.slice(0, 5) as f}<li>{fileName(f.path)} — {f.message}</li>{/each}
    </ul>
  </div>
{/if}

{#if tab === "organize"}
  <div class="mb-4 flex flex-wrap items-center gap-4">
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" bind:checked={byDate} class="h-4 w-4" />{$_("organizer.byDate")}
    </label>
    <button class={btn} onclick={preview} disabled={busy || !root}>{busy ? $_("organizer.scanning") : $_("organizer.preview")}</button>
  </div>

  {#if planned && plan.length === 0}
    <p class="text-slate-500">{$_("organizer.nothing")}</p>
  {:else if plan.length}
    <p class="mb-2 text-sm text-slate-500 dark:text-slate-400">{$_("organizer.previewHint")}</p>
    <div class="{card} mb-4 max-h-[48vh] overflow-y-auto">
      <table class="w-full text-sm">
        <tbody>
          {#each plan as m (m.src)}
            <tr class="border-b border-slate-100 last:border-0 dark:border-slate-700/60">
              <td class="w-8 px-3 py-2"><input type="checkbox" class="h-4 w-4" checked={picked.has(m.src)} onchange={() => (picked = toggle(picked, m.src))} /></td>
              <td class="max-w-[16rem] truncate py-2" title={m.src}>{fileName(m.src)}</td>
              <td class="px-2 text-slate-400">→</td>
              <td class="truncate py-2 pr-3 text-slate-600 dark:text-slate-300" title={m.dst}>{m.dst.slice(root.length + 1)}</td>
              <td class="whitespace-nowrap pr-3 text-right text-slate-400">{fmtSize(m.size)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <button class={btn} onclick={apply} disabled={busy || picked.size === 0}>{$_("organizer.apply", { values: { n: picked.size } })}</button>
  {/if}
{:else if tab === "dupes"}
  <div class="mb-4 flex flex-wrap items-center gap-4">
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" bind:checked={recursive} class="h-4 w-4" />{$_("organizer.recursive")}
    </label>
    <button class={btn} onclick={findDupes} disabled={busy || !root}>{busy ? $_("organizer.scanning") : $_("organizer.findDupes")}</button>
  </div>

  {#if scannedDupes && groups.length === 0}
    <p class="text-slate-500">{$_("organizer.noDupes")}</p>
  {:else if groups.length}
    <p class="mb-2 text-sm text-slate-500 dark:text-slate-400">
      {$_("organizer.dupeSummary", { values: { groups: groups.length, size: fmtSize(reclaimable) } })}
    </p>
    <div class="mb-4 flex max-h-[44vh] flex-col gap-3 overflow-y-auto">
      {#each groups as g (g.hash)}
        <div class="{card} p-3">
          <div class="mb-1 text-xs text-slate-400">{fmtSize(g.size)} × {g.files.length}</div>
          {#each g.files as f (f.path)}
            <div class="flex items-center gap-3 py-1 text-sm">
              <input type="checkbox" class="h-4 w-4" checked={removeSet.has(f.path)} onchange={() => toggleRemove(g, f.path)} />
              <span class="min-w-0 flex-1 truncate" title={f.path}>{f.path}</span>
              <span class="whitespace-nowrap text-xs text-slate-400">{f.modified}</span>
              {#if !removeSet.has(f.path)}
                <span class="rounded bg-emerald-100 px-2 py-0.5 text-xs text-emerald-800 dark:bg-emerald-500/20 dark:text-emerald-300">{$_("organizer.keep")}</span>
              {:else}
                <button class="text-xs text-indigo-600 underline dark:text-indigo-400" onclick={() => setKeeper(g, f.path)}>{$_("organizer.keep")}</button>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>

    {#if confirming}
      <div class="flex flex-wrap items-center gap-3 rounded-lg bg-amber-50 p-3 text-sm dark:bg-amber-500/10">
        <span>{$_("organizer.trashConfirm", { values: { n: removeSet.size } })}</span>
        <button class={btn} onclick={trashSelected} disabled={busy}>{$_("organizer.yes")}</button>
        <button class={ghost} onclick={() => (confirming = false)}>{$_("organizer.cancel")}</button>
      </div>
    {:else}
      <button class={btn} onclick={() => (confirming = true)} disabled={busy || removeSet.size === 0}>
        {$_("organizer.trash", { values: { n: removeSet.size } })}
      </button>
    {/if}
  {/if}
{:else}
  {#if history.length === 0}
    <p class="text-slate-500">{$_("organizer.historyEmpty")}</p>
  {:else}
    <div class="flex flex-col gap-2">
      {#each history as b (b.batchId)}
        <div class="{card} flex items-center gap-4 px-4 py-3 text-sm">
          <div class="flex-1">
            <div class="font-medium">{b.createdAt}</div>
            <div class="text-xs text-slate-400">{$_("organizer.files", { values: { n: b.total } })}</div>
          </div>
          {#if b.remaining > 0}
            <button class={ghost} onclick={() => undo(b.batchId)} disabled={busy}>{$_("organizer.undo")}</button>
          {:else}
            <span class="text-xs text-slate-400">{$_("organizer.allUndone")}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
{/if}
