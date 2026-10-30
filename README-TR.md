# Düzen-Baz

Dosyalarınızı, panonuzu ve göz sağlığınızı tek bir çatı altında toplayan, Türkçe arayüzlü, hafif bir masaüstü uygulaması. Tüm veriler yalnızca bilgisayarınızda kalır; internet bağlantısı gerekmez ve telemetri yoktur.

**Diller:** [English](README.md) · Türkçe

> **Windows, macOS ve Linux** üzerinde çalışır. Günlük kullanılan platform Windows'tur; macOS ve Linux her değişiklikte otomatik olarak derlenip test edilir, ama elle denenmemiştir. Bazı özellikler masaüstü oturumuna bağlıdır, özellikle Wayland'li Linux'ta ([Platform durumu](#platform-durumu)).

Aynı çekirdeği ve aynı yerel veriyi paylaşan iki programdan oluşur:

- **Düzen-Baz** — masaüstü uygulaması (Windows için kurulum paketleri; macOS ve Linux'ta kaynaktan derlersiniz, bkz. [Geliştirme](#geliştirme))
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
- `Duzen-Baz_<sürüm>_macos_x64.zip` — macOS uygulaması (Intel derlemesi; Apple Silicon'da Rosetta ile çalışır). Zip'i açıp `Duzen-Baz.app` dosyasını Uygulamalar'a sürükleyin
- `Duzen-Baz_<sürüm>_amd64.deb` — Linux paketi (Debian/Ubuntu, x86_64): `sudo apt install ./Duzen-Baz_<sürüm>_amd64.deb`
- `duzenbaz-cli_<sürüm>_linux_x64.tar.gz` — Linux için komut satırı aracı ve MCP sunucusu (x86_64)
- `duzenbaz-cli_<sürüm>_macos_x64.tar.gz` — macOS için komut satırı aracı ve MCP sunucusu (Intel derlemesi)
- `duzenbaz-cli.exe` — komut satırı aracı ve MCP sunucusu; kurulum gerektirmeyen tek bir taşınabilir dosya

> Paketler henüz kod imzalı değildir; Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.

macOS uygulaması imzalı veya noter onaylı değildir: ilk açılışta sağ tıklayıp **Aç**'ı seçin. Linux paketleri yalnızca x86_64 içindir; ARM'de (örneğin Raspberry Pi) kaynaktan derleyin ([Geliştirme](#geliştirme)).

## Geliştirme

Her sistem için: [Rust](https://rustup.rs) (stable) ve Node.js 20+. Ayrıca:

| Sistem | Başka ne gerekir |
|---|---|
| **Windows** | Visual Studio Build Tools (C++ iş yükü) ve MSVC Rust araç zinciri; WebView2 (Windows 11'de hazır gelir) |
| **macOS** | Xcode Komut Satırı Araçları: `xcode-select --install` (macOS 11 veya daha yeni) |
| **Linux** (Debian/Ubuntu) | `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev patchelf` |

Diğer Linux dağıtımları için [Tauri önkoşulları](https://v2.tauri.app/start/prerequisites/#linux) listesindeki karşılık gelen paketleri kullanın. Yukarıdaki Debian/Ubuntu satırı, projenin CI'ının kurduğu paketlerin aynısıdır.

```bash
npm install
npm run tauri dev      # masaüstü uygulamasını geliştirme modunda başlat
```

Testler ve denetimler:

```bash
cd src-tauri && cargo test --workspace     # Rust birim testleri (çekirdek, uygulama, CLI/MCP)
npm run check                              # Svelte / TypeScript denetimi
```

Paketleri üretmek için (çıktı `src-tauri/target/release/bundle` klasörüne yazılır):

```bash
npm run tauri build                   # bu sistemin desteklediği her şey
npm run tauri build -- --bundles deb  # veya yalnızca biri: deb, rpm, appimage (Linux); app, dmg (macOS); nsis, msi (Windows)
```

| Sistem | Paketler |
|---|---|
| Windows | `.exe` (NSIS) ve `.msi` |
| macOS | `.app` ve `.dmg` (CI `.app`'i derler) |
| Linux | `.deb`, `.rpm` ve `.AppImage` (CI `.deb`'i derler) |

Komut satırı aracını derlemek için (çıktı: `src-tauri/target/release/duzenbaz-cli`, Windows'ta `.exe` uzantılı):

```bash
cd src-tauri && cargo build --release -p duzenbaz-cli
```

İlk Windows derlemesinde paketleyici NSIS ve WiX araçlarını, Linux'ta AppImage araçlarını indirir (yalnızca derleme zamanında internet gerekir).

#### macOS için notlar

- Uygulama imzalı veya noter onaylı (notarized) değildir; bu yüzden Gatekeeper ilk açılışta engeller. Uygulamaya sağ tıklayıp **Aç**'ı seçin veya bir kez `xattr -dr com.apple.quarantine /yol/Duzen-Baz.app` çalıştırın.
- Pano penceresinden **otomatik yapıştırma**, öndeki uygulamaya Cmd+V gönderir; macOS buna yalnızca uygulamaya **Erişilebilirlik** izni verdikten sonra (Sistem Ayarları > Gizlilik ve Güvenlik) izin verir. İzin yoksa öğe yine panoya kopyalanır, yapıştırmayı siz yaparsınız.
- Tauri paketleyicisi, `com.duzenbaz.app` kimliğinin `.app` ile bittiği için bir uyarı yazdırır. Zararsızdır; veri klasörleri her sistemde aynı kalsın diye kimlik korunmuştur.

#### Linux için notlar

- Pano geçmişi, şifreleme anahtarını saklamak için bir **Secret Service** sağlayıcısına (GNOME Keyring veya KWallet) ihtiyaç duyar. Yoksa, sonradan çözemeyeceği veriyi saklamak yerine pano geçmişi kapalı kalır.
- Sistem tepsisi simgesi AppIndicator desteği ister. KDE ve çoğu masaüstünde vardır; GNOME'da "AppIndicator and KStatusNotifierItem Support" eklentisi gerekir.
- Burada varsayılan pano penceresi kısayolu **Ctrl+Alt+V**'dir (Ctrl+Shift+V terminallerde "yapıştır"dır). Pano sayfasından değiştirebilirsiniz.
- Gece modu, boşta kalma algısı, program algılama ve otomatik yapıştırma X11 kullanır. Uygulamaların bunları yapmasına izin vermeyen Wayland oturumlarında bulunmazlar (aşağıdaki tabloya bakın).

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
- **Pano verisi:** içerik ve etiketler SQLite'a yazılmadan önce ChaCha20-Poly1305 ile şifrelenir. Şifreleme anahtarı veritabanı dosyasında değil, işletim sisteminin güvenli deposunda (Windows Kimlik Bilgisi Yöneticisi, macOS Anahtar Zinciri, Linux Secret Service) saklanır. Öğe türü, boyut ve zaman gibi üst veriler şifrelenmez.
- Bu, **tam dosya şifrelemesi (SQLCipher) değildir.** SQLCipher'ın Windows'ta OpenSSL derlemesi gerektirmesi nedeniyle alan düzeyinde şifreleme tercih edilmiştir.
- Dosya işlemlerinde hedef yollar arayüzden değil Rust tarafında yeniden hesaplanır; silme işlemleri yalnızca Geri Dönüşüm Kutusu'na yapılır.
- Uygulama beklenmedik şekilde çökerse gece modu ekran rengini sıcak tonda bırakabilir; uygulamayı yeniden açıp gece modunu kapatmanız yeterlidir.
- MCP sunucusu çalıştığı klasörleri kısıtlar ve hiçbir şeyi kalıcı silmez; bkz. [Güvenlik seçenekleri](#mcp-sunucusu-olarak-kullanmak).

## Platform durumu

| Özellik | Windows | macOS | Linux (X11) | Linux (Wayland) |
|---|---|---|---|---|
| Dosya Düzenleyici, Toplu Adlandırıcı | evet | evet | evet | evet |
| Pano geçmişi (kaydetme, arama, sabitleme, etiket) | evet | evet | evet | evet, kompozitör data-control protokolünü sunuyorsa |
| Seçili programlardan kopyalananları yoksay | evet | evet | evet | hayır |
| Parola yöneticisi kopyalarını otomatik atla | evet | evet | hayır, yoksayma listesini kullanın | hayır |
| Açılır pencereden otomatik yapıştırma | evet | evet, Erişilebilirlik izni gerekir | evet | hayır |
| Genel açılır pencere kısayolu | evet | evet | evet | çoğunlukla hayır (kompozitörler engeller) |
| Mola hatırlatıcı | evet | evet | evet | evet, ama hep aktif sayılır (boşta kalma algısı yok) |
| Gece modu | evet | evet | evet | hayır |
| Komut satırı aracı ve MCP sunucusu | evet | evet | evet | evet |

Bunun nasıl denetlendiği: her değişiklikte bir GitHub Actions iş akışı Windows, macOS ve Linux'ta çalışır. Uygulamayı derler, tüm testleri çalıştırır, komut satırı aracını derleyip dener (MCP el sıkışması dahil), uygulamayı paketler (Linux'ta `.deb`, macOS'ta `.app`) ve çalışmaya devam ettiğini görmek için açar. Linux'ta ayrıca X11 kodunu sanal bir X sunucusuna karşı (boşta kalma süresi, sunucudan geri okunan gece modu renk değişikliği, gönderilen tuşlar) ve şifreleme anahtarını gerçek bir Secret Service'e (GNOME Keyring) karşı çalıştırır. macOS'ta CoreGraphics ve Objective-C kodunu (boşta kalma süresi, öndeki uygulama, gece modu) çalıştırır.

Gerçek bir masaüstü oturumu gerektirdiği için yalnızca Windows'ta elle denenenler: açılır pencere ve genel kısayolu, tepsi simgesi, başka bir uygulamaya otomatik yapıştırma, parola yöneticisi işaretleri ve canlı pano izleme. Sisteminizde farklı davranan bir şey görürseniz lütfen bildirin.

Ortak çekirdeğin ürettiği bazı mesajlar (örneğin dosya hataları) komut satırı aracında da Türkçedir.

## Lisans

[MIT](LICENSE) © 2026 BartuAbiHD

Simgeler [Lucide](https://lucide.dev) projesinden alınmıştır ve ISC lisansıyla dağıtılır; lisans metni `src/assets/icons/LICENSE-lucide.txt` dosyasındadır.
