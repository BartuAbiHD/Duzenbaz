# Otomatik güncelleme

Uygulama, **Ayarlar → Güncellemeler** bölümünden açılırsa GitHub'daki en son sürümü kontrol eder ve onayınızla kendini günceller.

- **Varsayılan kapalı.** Kapalıyken uygulama internete hiç bağlanmaz. Açıkken yalnızca `latest.json` dosyası istenir; hiçbir veri gönderilmez.
- **İmza doğrulaması.** Her güncelleme paketi, bu deponun `src-tauri/tauri.conf.json` dosyasındaki açık anahtarla doğrulanır. İmzası tutmayan paket kurulmaz.
- **Windows ve macOS:** indirir, kurar, yeniden başlatır. **Linux (.deb):** paket kendini kuramaz; uygulama yalnızca yeni sürümü bildirir ve indirme sayfasını açar.
- Kontrol, `https://github.com/BartuAbiHD/Duzenbaz/releases/latest/download/latest.json` adresine bakar. GitHub'ın "latest" adresi **ön sürümleri (pre-release) saymaz**; bu yüzden yayınlanan sürüm "Set as the latest release" olmalıdır.

## Yeni sürüm yayınlama (yayıncı için)

1. `package.json`, `src-tauri/Cargo.toml` ve `src-tauri/tauri.conf.json` içindeki sürümü artırın.
2. İmza anahtarını ortama verin (özel anahtar **depoya konmaz**; bu makinede `~/.tauri/duzenbaz.key`):

   ```bash
   export TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/duzenbaz.key"
   export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
   ```

3. Her sistemde, güncelleme paketlerini de üreterek derleyin:

   ```bash
   npm run tauri build -- --config '{"bundle":{"createUpdaterArtifacts":true}}'
   ```

   - Windows: `bundle/nsis/*-setup.exe` ve `.sig` dosyası
   - macOS: `npm run tauri build -- --bundles app --config '{"bundle":{"createUpdaterArtifacts":true}}'` → `bundle/macos/Duzen-Baz.app.tar.gz` ve `.sig`
   - Linux: `.deb` kendini güncelleyemez, ama sürümü bildirmek için imzalanmalıdır: `npx tauri signer sign -f ~/.tauri/duzenbaz.key Duzen-Baz_<sürüm>_amd64.deb`
4. Dosyaları Release'e yükleyin, sonra `latest.json` üretip onu da yükleyin:

   ```bash
   node scripts/make-update-manifest.mjs <sürüm> notlar.md out \
     windows-x86_64=Duzen-Baz_<sürüm>_x64-setup.exe \
     darwin-x86_64=Duzen-Baz_<sürüm>_macos_x64.app.tar.gz \
     linux-x86_64=Duzen-Baz_<sürüm>_amd64.deb
   gh release upload v<sürüm> out/latest.json
   ```

   Dosya adları Release'e yüklediğiniz adlarla aynı olmalıdır; `.sig` dosyaları yalnızca betiğin yanında okunur, Release'e yüklenmesi gerekmez.

> **Özel anahtarı kaybetmeyin ve paylaşmayın.** Kaybolursa mevcut kurulumlara güncelleme gönderilemez; sızarsa başkası güncelleme imzalayabilir.
