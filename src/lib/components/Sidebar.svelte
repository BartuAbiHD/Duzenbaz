<script lang="ts">
  import { _ } from "svelte-i18n";
  import Icon from "./Icon.svelte";

  type Item = { id: string; icon: string };
  let { items, current, onselect }: { items: Item[]; current: string; onselect: (id: string) => void } = $props();
</script>

<nav class="flex w-60 shrink-0 flex-col gap-1 border-r border-slate-200 bg-white p-3 dark:border-slate-700 dark:bg-slate-800">
  <div class="px-3 py-3 text-lg font-bold text-indigo-600 dark:text-indigo-400">{$_("app.name")}</div>
  {#each items as it (it.id)}
    <button
      class="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {current === it.id
        ? 'bg-indigo-50 font-semibold text-indigo-700 dark:bg-indigo-500/20 dark:text-indigo-300'
        : 'hover:bg-slate-100 dark:hover:bg-slate-700'} {it.id === 'settings' ? 'mt-auto' : ''}"
      onclick={() => onselect(it.id)}
    >
      <Icon name={it.icon} size={18} />
      {$_(`menu.${it.id}`)}
    </button>
  {/each}
</nav>
