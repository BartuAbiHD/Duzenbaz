# Düzen-Baz

Dosyalarınızı, panonuzu ve göz sağlığınızı tek bir çatı altında toplayan, Türkçe arayüzlü, hafif bir masaüstü uygulaması. Tüm veriler yalnızca bilgisayarınızda kalır; internet bağlantısı gerekmez ve telemetri yoktur.

**Diller:** [English](README.md) · Türkçe

> Şu an **Windows** için geliştirilmiş ve denenmiştir. macOS ve Linux için bazı özellikler henüz yoktur ([Platform durumu](#platform-durumu)).

Aynı çekirdeği ve aynı yerel veriyi paylaşan iki programdan oluşur:

- **Düzen-Baz** — masaüstü uygulaması (Windows kurulum paketi)
- **`duzenbaz-cli`** — komut satırı aracı; aynı zamanda bir **MCP sunucusu** olduğundan Claude gibi yapay zekâ asistanları da aynı özellikleri kullanabilir

## Modüller

### Dosya Düzenleyici
- Seçtiğiniz klasördeki (varsayılan: İndirilenler) dosyaları türüne (resim, belge, video, müzik, arşiv, program) ve isterseniz aya göre alt klasörlere ayırır.
- Taşımadan önce **ön izleme** gösterir; istemediğiniz dosyaları listeden çıkarabilirsiniz.
- Yapılan işlemleri **Geçmiş** sekmesinden tek tıkla geri alabilirsiniz. Hiçbir dosyanın üzerine yazılmaz, ad çakışmalarında `ad (1).uzantı` kullanılır.
- **Kopya dosyaları** içeriğine göre ([BLAKE3](https://github.com/BLAKE3-team/BLAKE3) özeti) bulur; onayınızla **Geri Dönüşüm Kutusu**'na gönderir. Kalıcı silme yoktur.

### Toplu Dosya Adlandırıcı
- Dosya veya klasörleri sürükleyip bırakın.
- Kurallar: tarih (fotoğraflar için **EXIF çekim tarihi** dahil), sıra numarası, önek/sonek, bul-değiştir, harf biçimi.
- Her değişiklikte **canlı ön izleme** (eski ad → yeni ad). Çakışan veya geçersiz adlar uygulanmadan önce işaretlenir.
- İşlem ya tamamen yapılır ya hiç yapılmaz; son işlem tek tıkla geri alınır.

### Pano Geçmişi Yöneticisi
- Arka planda panoyu dinler; metin, bağlantı ve görselleri kaydeder.
- **Ctrl+Shift+V** (ayarlanabilir) ile açılan, aranabilir küçük pencere. Seçtiğiniz öğe otomatik yapıştırılır.
- Öğeleri sabitleme, kategori ve etiket verme.
- Hassas içerik: parola yöneticilerinin "geçmişe alma" işaretini taşıyan kopyalar atlanır; ayrıca seçtiğiniz programlardan (varsayılan: KeePass, Bitwarden, 1Password vb.) kopyalananlar hiç kaydedilmez. Kaydı istediğiniz an duraklatabilirsiniz.
- Veriler yalnızca yerelde, **şifrelenmiş** olarak tutulur (bkz. [Güvenlik ve gizlilik](#güvenlik-ve-gizlilik)).

### Mola ve Göz Sağlığı Asistanı
- Ayarlanabilir mola hatırlatıcısı (varsayılan: 20 dakikada bir 20 saniye). Yalnızca bilgisayar başındayken sayar; yeterince uzun süre ayrılırsanız mola yapmış sayılırsınız.
- **Gece modu:** akşam saatlerinde ekran rengini sıcak tona çevirir, başlangıçta ve bitişte yarım saatlik yumuşak geçiş yapar (şimdilik yalnızca Windows).
- **Ekran süresi raporu:** bugünün toplamı, saat saat dağılım ve haftalık grafik. Yalnızca süre saklanır, hangi programları kullandığınız kaydedilmez.

### Genel
- Sistem tepsisinde çalışır; pencereyi kapatmak uygulamayı tepsiye indirir. İsteğe bağlı olarak Windows ile birlikte başlar.
- Açık / koyu / sistem teması, Türkçe ve İngilizce arayüz (çoklu dil altyapısı), 12/24 saat seçeneği.
- Hata durumlarında anlaşılır Türkçe mesajlar.

## Komut satırı aracı ve MCP sunucusu

`duzenbaz-cli.exe`, masaüstü uygulamasıyla aynı kodu ve aynı yerel veritabanını kullanır; birinde başlattığınız işlemi diğerinden geri alabilirsiniz. Masaüstü uygulaması çalışmıyorken de çalışır.

```bash
duzenbaz-cli organize plan   "C:\Users\ben\Downloads"            # önizleme, hiçbir şeyi değiştirmez
duzenbaz-cli organize apply  "C:\Users\ben\Downloads" --by-date  # taşımadan önce sorar
duzenbaz-cli organize undo   <batch-id>
duzenbaz-cli organize history

duzenbaz-cli duplicates find  "C:\Users\ben\Pictures" --recursive
duzenbaz-cli duplicates trash "C:\Users\ben\Pictures" --recursive --keep oldest   # Geri Dönüşüm Kutusu'na

duzenbaz-cli rename preview "C:\Fotograflar" --date exif --date-format %Y-%m-%d --number
duzenbaz-cli rename apply   "C:\Fotograflar" --prefix "Tatil_" --case title --yes
duzenbaz-cli rename undo    <batch-id>

duzenbaz-cli clipboard list -q fatura -n 10
duzenbaz-cli clipboard get 42 > not.txt
duzenbaz-cli screen-time
```

- Bir şeyi değiştiren komutlar onay ister; terminal yoksa `--yes` gerekir.
- Makine tarafından okunabilir çıktı için herhangi bir komuta `--json` ekleyin. `--data-dir <KLASÖR>` (veya `DUZENBAZ_DATA_DIR` ortam değişkeni) başka bir veri klasörü kullanır.
- Tüm seçenekler için `duzenbaz-cli --help` veya `duzenbaz-cli <komut> --help`.

### MCP sunucusu olarak kullanmak

`duzenbaz-cli mcp`, stdio üzerinden [Model Context Protocol](https://modelcontextprotocol.io) konuşur (protokol sürümleri 2025-06-18, 2025-03-26 ve 2024-11-05).

**Claude Code**

```bash
claude mcp add duzenbaz -- "C:\yol\duzenbaz-cli.exe" mcp --allow-dir "C:\Users\ben\Downloads"
```

**Claude Desktop** (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "duzenbaz": {
      "command": "C:\\yol\\duzenbaz-cli.exe",
      "args": ["mcp", "--allow-dir", "C:\\Users\\ben\\Downloads"]
    }
  }
}
```

Sunduğu araçlar:

| Araç | Ne yapar | Dosyaları değiştirir mi? |
|---|---|---|
| `organize_preview` | Tür klasörlerine nelerin taşınacağını gösterir | hayır |
| `organize_apply` | Dosyaları taşır, bir `batchId` döndürür | evet (geri alınabilir) |
| `organize_undo`, `organize_history` | İşlemi geri alır / işlemleri listeler | geri alma dosyaları değiştirir |
| `duplicates_find` | Aynı içerikli dosya gruplarını bulur | hayır |
| `duplicates_trash` | Her gruptan birini saklar, kalanları Geri Dönüşüm Kutusu'na gönderir | evet (Geri Dönüşüm Kutusu) |
| `rename_preview` | Eski → yeni adları gösterir, çakışmaları işaretler | hayır |
| `rename_apply` | Tüm dosyaları yeniden adlandırır ya da hiçbirini, `batchId` döndürür | evet (geri alınabilir) |
| `rename_undo`, `rename_history` | Yeniden adlandırmayı geri alır / listeler | geri alma dosyaları değiştirir |
| `screen_time_report` | Ekran süresi ve mola istatistikleri | hayır |
| `clipboard_search` | Pano geçmişinde arama yapar (**varsayılan olarak kapalı**) | hayır |

Güvenlik seçenekleri:

- **Klasörler:** araçlar yalnızca izin verilen klasörlerde çalışır. Varsayılan İndirilenler klasörünüzdür; başkalarına izin vermek için `--allow-dir <KLASÖR>` (tekrarlanabilir) kullanın. `--allow-any-path` kısıtlamayı kaldırır; bağlı yapay zekâ o zaman sizin erişebildiğiniz her dosyaya dokunabileceği için dikkatli kullanın.
- **`--read-only`:** yalnızca hiçbir şeyi değiştirmeyen araçlar sunulur.
- **`--allow-clipboard`:** pano geçmişi parola veya özel metin içerebileceği için aracı siz istemedikçe gizlidir.
- Hiçbir şey kalıcı silinmez, her değişiklik geri alınabilir ve araç açıklamaları asistana önce önizleme yapmasını, uygulamadan önce sizden onay almasını söyler.

## Teknoloji

| Katman | Seçim |
|---|---|
| Çekirdek | Rust; ortak kütüphane (`duzenbaz-core`), masaüstü uygulaması ve CLI olarak ayrılmıştır |
| Masaüstü kabuğu | [Tauri 2](https://tauri.app) |
| Arayüz | Svelte 5 + Tailwind CSS 4 + Vite |
| Yerel veri | SQLite (`rusqlite`) |
| Simgeler | [Lucide](https://lucide.dev) (projede SVG dosyası olarak durur, çevrimdışı çalışır) |

**Neden bu seçimler?** Tauri, Electron'a göre çok daha küçük kurulum paketi (~3 MB) ve düşük bellek kullanımı sunar; bu da tepside sürekli çalışan bir uygulama için önemlidir. Svelte, React'e göre daha az kodla daha küçük bir paket üretir ve tema/dil gibi durumları basit tutar.

Kullanılan başlıca Rust paketleri: `walkdir`, `blake3`, `trash`, `kamadak-exif`, `arboard`, `clipboard-master`, `rusqlite`, `chacha20poly1305`, `keyring`, `clap`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`.

## Kurulum (kullanıcılar için)

[Releases](https://github.com/BartuAbiHD/Duzenbaz/releases) sayfasından indirin:

- `Duzen-Baz_<sürüm>_x64-setup.exe` — kullanıcı bazında kurulur, yönetici izni gerekmez
- `Duzen-Baz_<sürüm>_x64_tr-TR.msi` — MSI paketi
- `duzenbaz-cli.exe` — komut satırı aracı ve MCP sunucusu; kurulum gerektirmeyen tek bir taşınabilir dosya

> Paketler henüz kod imzalı değildir; Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.

## Geliştirme

Gereksinimler:

- [Rust](https://rustup.rs) (stable, MSVC araç zinciri)
- Node.js 20+
- Visual Studio Build Tools (C++ iş yükü) ve WebView2 (Windows 11'de hazır gelir)

```bash
npm install
npm run tauri dev      # masaüstü uygulamasını geliştirme modunda başlat
```

Testler ve denetimler:

```bash
cd src-tauri && cargo test --workspace     # Rust birim testleri (çekirdek, uygulama, CLI/MCP)
npm run check                              # Svelte / TypeScript denetimi
```

Kurulum paketlerini üretmek için (çıktı `src-tauri/target/release/bundle` klasörüne yazılır):

```bash
npm run tauri build
```

Komut satırı aracını derlemek için (çıktı: `src-tauri/target/release/duzenbaz-cli.exe`):

```bash
cd src-tauri && cargo build --release -p duzenbaz-cli
```

İlk paket derlemesinde paketleyici NSIS ve WiX araçlarını indirir (yalnızca derleme zamanında internet gerekir).

Tarayıcı kısayolları (F12, Ctrl+Shift+I, sağ tık menüsü) bilerek kapalıdır. Hata ayıklamak için uygulamayı `DUZENBAZ_DEVTOOLS=1` ortam değişkeniyle başlatın; bu, Windows tarafındaki kilidi atlar.

## Proje yapısı

```
src/                        Svelte arayüzü
  modules/                  organizer, renamer, clipboard, breaks, settings
  lib/                      ortak bileşenler, depolar, i18n (tr.json, en.json)
  assets/icons/             SVG simgeler (Lucide)
src-tauri/
  src/                      masaüstü uygulaması: Tauri komutları, tepsi, açılır pencereler, pano dinleyicisi
  crates/core/              ortak çekirdek, Tauri'ye bağlı değil
    src/organizer/          tarama, sınıflandırma, kopya tespiti, geri alma
    src/renamer/            kurallar, EXIF, güvenli yeniden adlandırma
    src/clipboard/          şifreli depo, platform yardımcıları
    src/breaks/             durum makinesi, gece modu, ekran süresi
    src/db.rs, history.rs   sürümlü şema geçişli SQLite bağlantısı, ortak geri alma günlüğü
  crates/cli/               duzenbaz-cli: komut satırı ve MCP sunucusu
```

Yeni bir dil eklemek için `src/lib/i18n/` altına bir JSON dosyası ekleyip `index.ts` içine kaydetmeniz yeterlidir.

## Güvenlik ve gizlilik

- Masaüstü uygulaması internete bağlanmaz; içerik güvenlik politikası (CSP) ağ erişimini kapalı tutar. Telemetri yoktur. Komut satırı aracı da ağı kullanmaz.
- **Pano verisi:** içerik ve etiketler SQLite'a yazılmadan önce ChaCha20-Poly1305 ile şifrelenir. Şifreleme anahtarı veritabanı dosyasında değil, işletim sisteminin güvenli deposunda (Windows Kimlik Bilgisi Yöneticisi) saklanır. Öğe türü, boyut ve zaman gibi üst veriler şifrelenmez.
- Bu, **tam dosya şifrelemesi (SQLCipher) değildir.** SQLCipher'ın Windows'ta OpenSSL derlemesi gerektirmesi nedeniyle alan düzeyinde şifreleme tercih edilmiştir.
- Dosya işlemlerinde hedef yollar arayüzden değil Rust tarafında yeniden hesaplanır; silme işlemleri yalnızca Geri Dönüşüm Kutusu'na yapılır.
- Uygulama beklenmedik şekilde çökerse gece modu ekran rengini sıcak tonda bırakabilir; uygulamayı yeniden açıp gece modunu kapatmanız yeterlidir.
- MCP sunucusu çalıştığı klasörleri kısıtlar ve hiçbir şeyi kalıcı silmez; bkz. [Güvenlik seçenekleri](#mcp-sunucusu-olarak-kullanmak).

## Platform durumu

| Özellik | Windows | macOS / Linux |
|---|---|---|
| Dosya Düzenleyici, Toplu Adlandırıcı | evet | derlenmesi beklenir, denenmedi |
| Pano geçmişi | evet | dinleme beklenir; parola yöneticisi işareti ve otomatik yapıştırma yok |
| Mola hatırlatıcı | evet | boşta kalma algısı yok (hep aktif sayılır) |
| Gece modu | evet | henüz yok |
| Komut satırı aracı ve MCP sunucusu | evet | derlenmesi beklenir, denenmedi |

Ortak çekirdeğin ürettiği bazı mesajlar (örneğin dosya hataları) komut satırı aracında da Türkçedir.

## Lisans

[MIT](LICENSE) © 2026 BartuAbiHD

Simgeler [Lucide](https://lucide.dev) projesinden alınmıştır ve ISC lisansıyla dağıtılır; lisans metni `src/assets/icons/LICENSE-lucide.txt` dosyasındadır.
