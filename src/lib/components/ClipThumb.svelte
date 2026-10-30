<script lang="ts">
  import { onMount } from "svelte";
  import { call } from "../api";

  let { id, size = 56 }: { id: number; size?: number } = $props();
  let src = $state("");

  onMount(async () => {
    try {
      src = await call<string>("clip_image", { id });
    } catch {}
  });
</script>

{#if src}
  <img {src} alt="" class="rounded object-cover" style="width:{size}px;height:{size}px" />
{:else}
  <div class="rounded bg-slate-200 dark:bg-slate-700" style="width:{size}px;height:{size}px"></div>
{/if}
