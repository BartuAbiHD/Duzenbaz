// latest.json üretir: uygulamanın kendini güncellemek için okuduğu dosya.
// Kullanım: node scripts/make-update-manifest.mjs <sürüm> <notlar.md> <çıktı-klasörü> [platform=dosya ...]
// Her dosyanın yanında, "tauri build" veya "tauri signer sign" ile üretilmiş <dosya>.sig bulunmalıdır.
// Örnek:
//   node scripts/make-update-manifest.mjs 0.2.0 notes.md out \
//     windows-x86_64=Duzen-Baz_0.2.0_x64-setup.exe \
//     darwin-x86_64=Duzen-Baz_0.2.0_macos_x64.app.tar.gz \
//     linux-x86_64=Duzen-Baz_0.2.0_amd64.deb
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { basename } from "node:path";

const [version, notesFile, outDir, ...entries] = process.argv.slice(2);
if (!version || !notesFile || !outDir || entries.length === 0) {
  console.error("Kullanım: node scripts/make-update-manifest.mjs <sürüm> <notlar.md> <çıktı-klasörü> platform=dosya ...");
  process.exit(1);
}

const base = `https://github.com/BartuAbiHD/Duzenbaz/releases/download/v${version}`;
const platforms = {};
for (const e of entries) {
  const [platform, file] = e.split("=");
  platforms[platform] = {
    signature: readFileSync(`${file}.sig`, "utf8").trim(),
    url: `${base}/${encodeURIComponent(basename(file))}`,
  };
}
// Apple Silicon, Intel derlemesini Rosetta ile çalıştırır; ayrı ARM paketi yoksa aynı dosya kullanılır.
if (platforms["darwin-x86_64"] && !platforms["darwin-aarch64"]) platforms["darwin-aarch64"] = platforms["darwin-x86_64"];

const manifest = { version, notes: readFileSync(notesFile, "utf8").trim(), pub_date: new Date().toISOString(), platforms };
mkdirSync(outDir, { recursive: true });
writeFileSync(`${outDir}/latest.json`, JSON.stringify(manifest, null, 2) + "\n");
console.log(`${outDir}/latest.json yazıldı:`, Object.keys(platforms).join(", "));
