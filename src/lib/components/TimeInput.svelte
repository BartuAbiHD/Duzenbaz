<script lang="ts">
  // Tarayıcının saat kutusu işletim sisteminin bölgesine bağlı olduğundan kendi saat seçicimizi kullanıyoruz.
  // Gösterim Ayarlar'daki saat biçimine uyar (24 saat veya AM/PM); değer her zaman 24 saatlik "SS:DD" olarak tutulur.
  import { timeFormat } from "../stores/timeFormat";

  let { value = $bindable("00:00"), onchange }: { value: string; onchange?: () => void } = $props();

  const pad = (n: number) => String(n).padStart(2, "0");
  const hours24 = Array.from({ length: 24 }, (_, i) => pad(i));
  const hours12 = Array.from({ length: 12 }, (_, i) => pad(i + 1));

  const h24 = $derived(Number(value.split(":")[0] ?? 0));
  const hh = $derived(pad(h24));
  const mm = $derived(value.split(":")[1] ?? "00");
  const is12 = $derived($timeFormat === "12");
  const period = $derived(h24 >= 12 ? "PM" : "AM");
  const hh12 = $derived(pad(h24 % 12 || 12));
  // 5'er dakika aralıklı liste; kayıtlı değer bu aralıkta değilse o da listeye eklenir.
  const minutes = $derived([...new Set([...Array.from({ length: 12 }, (_, i) => pad(i * 5)), mm])].sort());

  function set(h: string, m: string) {
    value = `${h}:${m}`;
    onchange?.();
  }

  function set12(hour12: string, per: string) {
    const h = (Number(hour12) % 12) + (per === "PM" ? 12 : 0);
    set(pad(h), mm);
  }

  const select = "rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800";
</script>

<div class="flex items-center gap-1.5">
  {#if is12}
    <select class={select} value={hh12} onchange={(e) => set12(e.currentTarget.value, period)} aria-label="Saat">
      {#each hours12 as h}<option value={h}>{h}</option>{/each}
    </select>
  {:else}
    <select class={select} value={hh} onchange={(e) => set(e.currentTarget.value, mm)} aria-label="Saat">
      {#each hours24 as h}<option value={h}>{h}</option>{/each}
    </select>
  {/if}
  <span class="text-slate-400">:</span>
  <select class={select} value={mm} onchange={(e) => set(hh, e.currentTarget.value)} aria-label="Dakika">
    {#each minutes as m}<option value={m}>{m}</option>{/each}
  </select>
  {#if is12}
    <select class={select} value={period} onchange={(e) => set12(hh12, e.currentTarget.value)} aria-label="AM/PM">
      <option value="AM">AM</option>
      <option value="PM">PM</option>
    </select>
  {/if}
</div>
