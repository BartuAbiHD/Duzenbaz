<script lang="ts">
  import { _ } from "svelte-i18n";
  let { id }: { id: string } = $props();
</script>

<h1 class="mb-2 text-2xl font-bold">{$_(`menu.${id}`)}</h1>
<p class="text-slate-500 dark:text-slate-400">{$_("common.soon")}</p>
