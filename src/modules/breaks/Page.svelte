<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { _, locale } from "svelte-i18n";
  import { listen } from "@tauri-apps/api/event";
  import { call } from "../../lib/api";
  import { notify } from "../../lib/stores/toast";
  import { formatDuration, type BreakSettings, type BreakState, type Report } from "../../lib/breaks";
  import BarChart from "../../lib/components/BarChart.svelte";
  import TimeInput from "../../lib/components/TimeInput.svelte";

  let settings = $state<BreakSettings | null>(null);
  let nightSupported = $state(true);
  let live = $state<BreakState | null>(null);
  let report = $state<Report | null>(null);
  const unlisten: Array<() => void> = [];
  let timer: ReturnType<typeof setInterval>;

  const todaySecs = $derived(report ? report.hours.reduce((a, b) => a + b, 0) : 0);
  const untilBreak = $derived(live ? Math.max(0, live.intervalSecs - live.workSecs) : 0);
  const workPct = $derived(live && live.intervalSecs ? Math.min(100, (live.workSecs / live.intervalSecs) * 100) : 0);
  const dayLabels = $derived(report ? report.days.map((d) => new Date(d.day + "T12:00:00").toLocaleDateString($locale ?? "tr", { weekday: "short" })) : []);
  const hourLabels = Array.from({ length: 24 }, (_, i) => String(i));

  // Türkiye'de hafta Pazartesi'den başlar (1); diğer dillerde Pazar (0).
  const weekStart = $derived(($locale ?? "tr").startsWith("tr") ? 1 : 0);

  async function loadReport() {
    try {
      report = await call<Report>("breaks_report", { weekStart });
    } catch {}
  }

  // Dil değişince hafta başlangıcı da değişeceği için rapor yeniden yüklenir.
  $effect(() => {
    weekStart;
    loadReport();
  });

  async function save() {
    if (!settings) return;
    try {
      const r = await call<{ settings: BreakSettings; nightSupported: boolean }>("breaks_set_settings", { settings });
      settings = r.settings;
    } catch {}
  }

  async function preview() {
    if (!settings) return;
    try {
      await call("breaks_preview_night", { warmth: settings.night.warmth });
      notify($_("breaks.previewing"));
    } catch {}
  }

  onMount(async () => {
    try {
      const r = await call<{ settings: BreakSettings; nightSupported: boolean }>("breaks_get_settings");
      settings = r.settings;
      nightSupported = r.nightSupported;
      live = await call<BreakState>("breaks_get_state");
    } catch {}
    timer = setInterval(loadReport, 30000);
    unlisten.push(await listen<BreakState>("breaks-state", (e) => (live = e.payload)));
  });
  onDestroy(() => {
    clearInterval(timer);
    unlisten.forEach((u) => u());
  });

  const input = "rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800";
  const card = "rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-700 dark:bg-slate-800";
  const btn = "rounded-lg border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-100 dark:border-slate-600 dark:hover:bg-slate-700";
  const label = "mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400";
</script>

<h1 class="mb-4 text-2xl font-bold">{$_("breaks.title")}</h1>

{#if settings}
  <div class="flex max-w-3xl flex-col gap-4">
    <section class={card}>
      <div class="mb-3 flex items-center justify-between">
        <label class="flex items-center gap-2 text-sm font-semibold">
          <input type="checkbox" bind:checked={settings.enabled} onchange={save} />{$_("breaks.reminder")}
        </label>
        <button class={btn} onclick={() => call("breaks_start_now")} disabled={!settings.enabled}>{$_("breaks.startNow")}</button>
      </div>
      {#if settings.enabled && live}
        <div class="mb-1 flex justify-between text-xs text-slate-500">
          <span>{$_("breaks.nextBreak")}</span>
          <span class="tabular-nums">{Math.floor(untilBreak / 60)}:{String(untilBreak % 60).padStart(2, "0")}</span>
        </div>
        <div class="mb-4 h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
          <div class="h-full rounded-full bg-indigo-500 transition-[width] duration-1000 ease-linear" style="width:{workPct}%"></div>
        </div>
      {/if}
      <div class="grid grid-cols-2 gap-3 sm:max-w-sm">
        <label><span class={label}>{$_("breaks.every")}</span><input type="number" min="1" max="180" class="{input} w-full" bind:value={settings.intervalMin} onchange={save} /></label>
        <label><span class={label}>{$_("breaks.lasts")}</span><input type="number" min="5" max="900" class="{input} w-full" bind:value={settings.breakSecs} onchange={save} /></label>
      </div>
      <p class="mt-3 text-xs text-slate-400">{$_("breaks.idleNote")}</p>
    </section>

    <section class={card}>
      <label class="flex items-center gap-2 text-sm font-semibold">
        <input type="checkbox" bind:checked={settings.night.enabled} onchange={save} disabled={!nightSupported} />{$_("breaks.night")}
      </label>
      <p class="mt-1 text-xs text-slate-400">{$_("breaks.nightHint")}</p>
      {#if !nightSupported}
        <p class="mt-2 text-xs text-amber-600">{$_("breaks.nightUnsupported")}</p>
      {:else}
        <div class="mt-3 grid grid-cols-2 gap-3 sm:max-w-sm">
          <div><span class={label}>{$_("breaks.from")}</span><TimeInput bind:value={settings.night.start} onchange={save} /></div>
          <div><span class={label}>{$_("breaks.to")}</span><TimeInput bind:value={settings.night.end} onchange={save} /></div>
        </div>
        <label class="mt-3 block sm:max-w-sm">
          <span class={label}>{$_("breaks.warmth")}: {settings.night.warmth}%</span>
          <input type="range" min="0" max="100" class="w-full" bind:value={settings.night.warmth} onchange={save} />
        </label>
        <button class="{btn} mt-3" onclick={preview}>{$_("breaks.preview")}</button>
      {/if}
    </section>

    <section class={card}>
      <div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
        <div class="text-sm font-semibold">{$_("breaks.report")}</div>
        <div class="text-sm text-slate-500">
          {$_("breaks.today")}: <span class="font-semibold text-slate-900 dark:text-slate-100">{formatDuration(todaySecs)}</span>
          · {$_("breaks.taken", { values: { n: report?.breaksTaken ?? 0 } })}
          · {$_("breaks.skipped", { values: { n: report?.breaksSkipped ?? 0 } })}
        </div>
      </div>
      {#if report}
        <div class="mb-1 text-xs text-slate-400">{$_("breaks.week")}</div>
        <BarChart values={report.days.map((d) => d.seconds)} labels={dayLabels} highlight={report.todayIndex} format={formatDuration} />
        <div class="mb-1 mt-4 text-xs text-slate-400">{$_("breaks.byHour")}</div>
        <BarChart values={report.hours} labels={hourLabels} highlight={new Date().getHours()} labelEvery={3} format={formatDuration} />
      {/if}
      <p class="mt-3 text-xs text-slate-400">{$_("breaks.privacy")}</p>
    </section>
  </div>
{/if}
