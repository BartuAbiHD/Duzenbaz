<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { listen } from "@tauri-apps/api/event";
  import { call } from "../../lib/api";
  import type { BreakState } from "../../lib/breaks";

  let live = $state<BreakState | null>(null);
  let unlisten: (() => void) | undefined;

  const R = 44;
  const C = 2 * Math.PI * R;
  const progress = $derived(live && live.breakSecs ? live.breakRemaining / live.breakSecs : 0);

  onMount(async () => {
    try {
      live = await call<BreakState>("breaks_get_state");
    } catch {}
    unlisten = await listen<BreakState>("breaks-state", (e) => (live = e.payload));
  });
  onDestroy(() => unlisten?.());
</script>

<div class="flex h-full flex-col items-center justify-center gap-3 border border-slate-300 bg-white px-6 text-center dark:border-slate-600 dark:bg-slate-900">
  <div class="text-sm font-semibold text-indigo-600 dark:text-indigo-400">{$_("breaks.breakTitle")}</div>
  <div class="relative h-28 w-28">
    <svg viewBox="0 0 100 100" class="-rotate-90">
      <circle cx="50" cy="50" r={R} fill="none" stroke-width="6" class="stroke-slate-200 dark:stroke-slate-700" />
      <circle cx="50" cy="50" r={R} fill="none" stroke-width="6" stroke-linecap="round" stroke-dasharray={C} stroke-dashoffset={C * (1 - progress)} class="stroke-indigo-500 transition-[stroke-dashoffset] duration-1000 ease-linear" />
    </svg>
    <div class="absolute inset-0 flex items-center justify-center text-3xl font-bold tabular-nums">{live?.breakRemaining ?? 0}</div>
  </div>
  <p class="text-xs text-slate-500 dark:text-slate-400">{$_("breaks.breakTip")}</p>
  <div class="flex gap-2">
    <button class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700" onclick={() => call("breaks_snooze", { minutes: 5 })}>{$_("breaks.snooze")}</button>
    <button class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700" onclick={() => call("breaks_skip")}>{$_("breaks.skip")}</button>
  </div>
</div>
