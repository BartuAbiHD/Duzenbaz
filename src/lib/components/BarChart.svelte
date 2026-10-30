<script lang="ts">
  // Basit çubuk grafik. `values` saniye cinsindendir; ekranda dakika/saat olarak yazılır.
  type Props = { values: number[]; labels: string[]; highlight?: number; labelEvery?: number; format: (secs: number) => string };
  let { values, labels, highlight = -1, labelEvery = 1, format }: Props = $props();

  const W = 600;
  const H = 150;
  const PAD_TOP = 18;
  const PAD_BOTTOM = 22;
  const max = $derived(Math.max(...values, 60));
  const slot = $derived(W / values.length);
  const barH = (v: number) => (v / max) * (H - PAD_TOP - PAD_BOTTOM);
</script>

<svg viewBox="0 0 {W} {H}" class="w-full" role="img">
  <text x="0" y="11" class="fill-slate-400 text-[11px]">{format(max)}</text>
  <line x1="0" x2={W} y1={H - PAD_BOTTOM} y2={H - PAD_BOTTOM} class="stroke-slate-300 dark:stroke-slate-600" />
  {#each values as v, i}
    <g>
      <title>{labels[i]}: {format(v)}</title>
      <rect
        x={i * slot + slot * 0.15}
        y={H - PAD_BOTTOM - barH(v)}
        width={slot * 0.7}
        height={Math.max(barH(v), v > 0 ? 2 : 0)}
        rx="3"
        class={i === highlight ? "fill-indigo-500" : "fill-indigo-300 dark:fill-indigo-400/50"}
      />
      {#if i % labelEvery === 0}
        <text x={i * slot + slot / 2} y={H - 6} text-anchor="middle" class="fill-slate-500 text-[11px] dark:fill-slate-400">{labels[i]}</text>
      {/if}
    </g>
  {/each}
</svg>
