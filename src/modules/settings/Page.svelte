<script lang="ts">
  import { onMount } from "svelte";
  import { _, locale } from "svelte-i18n";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { call } from "../../lib/api";
  import { theme, type Theme } from "../../lib/stores/theme";
  import { timeFormat, type TimeFormat } from "../../lib/stores/timeFormat";
  import { LOCALES, setLocale } from "../../lib/i18n";
  import { notify } from "../../lib/stores/toast";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { autoCheck, updateState, checkForUpdate, installUpdate, restartApp } from "../../lib/stores/updates";

  const RELEASES = "https://github.com/BartuAbiHD/Duzenbaz/releases";
  let canSelfUpdate = $state(true);

  let autostart = $state(false);
  let dbVersion = $state<number | null>(null);

  onMount(async () => {
    try {
      autostart = await isEnabled();
    } catch {
      notify("Otomatik başlatma durumu okunamadı.", "error");
    }
    try {
      dbVersion = await call<number>("db_version");
    } catch {}
    try {
      canSelfUpdate = await call<boolean>("app_can_self_update");
    } catch {}
  });

  async function toggleAutostart() {
    try {
      if (autostart) await disable();
      else await enable();
      autostart = !autostart;
    } catch {
      notify("Otomatik başlatma ayarı değiştirilemedi.", "error");
    }
  }

  const timeFormats: { id: TimeFormat; key: string }[] = [
    { id: "24", key: "settings.time24" },
    { id: "12", key: "settings.time12" },
  ];

  const themes: { id: Theme; key: string }[] = [
    { id: "system", key: "settings.themeSystem" },
    { id: "light", key: "settings.themeLight" },
    { id: "dark", key: "settings.themeDark" },
  ];
</script>

<h1 class="mb-6 text-2xl font-bold">{$_("settings.title")}</h1>

<div class="flex max-w-xl flex-col gap-6">
  <section>
    <div class="mb-2 text-sm font-semibold">{$_("settings.theme")}</div>
    <div class="inline-flex overflow-hidden rounded-lg border border-slate-300 dark:border-slate-600">
      {#each themes as t (t.id)}
        <button
          class="px-4 py-2 text-sm {$theme === t.id ? 'bg-indigo-600 text-white' : 'hover:bg-slate-100 dark:hover:bg-slate-700'}"
          onclick={() => theme.set(t.id)}>{$_(t.key)}</button
        >
      {/each}
    </div>
  </section>

  <section>
    <label class="mb-2 block text-sm font-semibold" for="lang">{$_("settings.language")}</label>
    <select
      id="lang"
      class="rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800"
      value={$locale}
      onchange={(e) => setLocale(e.currentTarget.value)}
    >
      {#each LOCALES as l (l.code)}<option value={l.code}>{l.label}</option>{/each}
    </select>
  </section>

  <section>
    <div class="mb-2 text-sm font-semibold">{$_("settings.timeFormat")}</div>
    <div class="inline-flex overflow-hidden rounded-lg border border-slate-300 dark:border-slate-600">
      {#each timeFormats as t (t.id)}
        <button
          class="px-4 py-2 text-sm {$timeFormat === t.id ? 'bg-indigo-600 text-white' : 'hover:bg-slate-100 dark:hover:bg-slate-700'}"
          onclick={() => timeFormat.set(t.id)}>{$_(t.key)}</button
        >
      {/each}
    </div>
  </section>

  <section>
    <label class="flex items-center gap-3 text-sm">
      <input type="checkbox" checked={autostart} onchange={toggleAutostart} class="h-4 w-4" />
      <span class="font-semibold">{$_("settings.autostart")}</span>
    </label>
    <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">{$_("settings.autostartHint")}</p>
  </section>

  <section>
    <div class="mb-2 text-sm font-semibold">{$_("settings.updates")}</div>
    <label class="flex items-center gap-3 text-sm">
      <input type="checkbox" bind:checked={$autoCheck} class="h-4 w-4" />
      <span>{$_("settings.updatesAuto")}</span>
    </label>
    <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">{$_("settings.updatesHint")}</p>

    <div class="mt-3 flex flex-wrap items-center gap-3 text-sm">
      <button
        class="rounded-lg border border-slate-300 px-4 py-2 hover:bg-slate-100 disabled:opacity-50 dark:border-slate-600 dark:hover:bg-slate-700"
        disabled={$updateState.status === "checking" || $updateState.status === "downloading"}
        onclick={() => checkForUpdate(false)}>{$_("settings.updatesCheck")}</button
      >
      {#if $updateState.status === "checking"}
        <span>{$_("settings.updatesChecking")}</span>
      {:else if $updateState.status === "none"}
        <span>{$_("settings.updatesNone")}</span>
      {:else if $updateState.status === "error"}
        <span class="text-rose-600 dark:text-rose-400">{$_("settings.updatesError")}</span>
      {:else if $updateState.status === "downloading"}
        <span>{$_("settings.updatesDownloading", { values: { percent: $updateState.percent ?? 0 } })}</span>
      {:else if $updateState.status === "ready"}
        <span>{$_("settings.updatesReady")}</span>
        <button class="rounded-lg bg-indigo-600 px-4 py-2 text-white" onclick={restartApp}>{$_("settings.updatesRestart")}</button>
      {/if}
    </div>

    {#if $updateState.status === "available"}
      <div class="mt-3 rounded-lg border border-indigo-300 bg-indigo-50 p-3 text-sm dark:border-indigo-500/40 dark:bg-indigo-500/10">
        <div class="font-semibold">{$_("settings.updatesAvailable", { values: { version: $updateState.version ?? "" } })}</div>
        {#if $updateState.notes}
          <p class="mt-1 whitespace-pre-line text-xs text-slate-600 dark:text-slate-300">{$updateState.notes}</p>
        {/if}
        {#if canSelfUpdate}
          <button class="mt-3 rounded-lg bg-indigo-600 px-4 py-2 text-white" onclick={installUpdate}>{$_("settings.updatesInstall")}</button>
        {:else}
          <p class="mt-2 text-xs">{$_("settings.updatesManual")}</p>
          <button class="mt-2 rounded-lg bg-indigo-600 px-4 py-2 text-white" onclick={() => openUrl(RELEASES)}>{$_("settings.updatesOpenPage")}</button>
        {/if}
      </div>
    {/if}
  </section>

  <section class="text-sm">
    <span class="font-semibold">{$_("settings.database")}:</span>
    {dbVersion ?? "—"}
  </section>

  <p class="rounded-lg bg-emerald-50 p-3 text-xs text-emerald-800 dark:bg-emerald-500/10 dark:text-emerald-300">
    {$_("settings.privacy")}
  </p>
</div>
