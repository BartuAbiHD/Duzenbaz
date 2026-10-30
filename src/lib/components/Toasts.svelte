<script lang="ts">
  import { toasts } from "../stores/toast";
</script>

<div class="fixed bottom-4 right-4 z-50 flex max-w-sm flex-col gap-2">
  {#each $toasts as t (t.id)}
    <div
      role="alert"
      class="rounded-lg px-4 py-3 text-sm shadow-lg {t.kind === 'error'
        ? 'bg-red-600 text-white'
        : 'bg-slate-800 text-white dark:bg-slate-200 dark:text-slate-900'}"
    >
      {t.text}
    </div>
  {/each}
</div>
