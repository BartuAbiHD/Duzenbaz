<script lang="ts">
  // Simgeler src/assets/icons içindeki SVG dosyalarıdır (Lucide, ISC lisansı); pakete gömülür, internet gerekmez.
  const files = import.meta.glob("../../assets/icons/*.svg", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
  const icons = Object.fromEntries(Object.entries(files).map(([path, svg]) => [path.split("/").pop()!.replace(".svg", ""), svg]));

  let { name, size = 18, class: cls = "" }: { name: string; size?: number; class?: string } = $props();

  const svg = $derived(
    (icons[name] ?? "")
      .replace(/<!--.*?-->/s, "")
      .replace(/\swidth="24"/, ` width="${size}"`)
      .replace(/\sheight="24"/, ` height="${size}"`),
  );
</script>

<span class="inline-flex shrink-0 {cls}" aria-hidden="true">{@html svg}</span>
