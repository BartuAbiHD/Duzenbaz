# Düzen-Baz

Dosyalarınızı, panonuzu ve göz sağlığınızı tek bir çatı altında toplayan, Türkçe arayüzlü, hafif bir masaüstü uygulaması. Tüm veriler yalnızca bilgisayarınızda kalır; internet bağlantısı gerekmez ve telemetri yoktur.

> Şu an **Windows** için geliştirilmiş ve denenmiştir. macOS ve Linux için bazı özellikler henüz yoktur (aşağıya bakın).

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
- **Gece modu:** akşam saatlerinde ekran rengini sıcak tona çevirir, başlangıç ve bitişte yarım saatlik yumuşak geçiş yapar (şimdilik yalnızca Windows).
- **Ekran süresi raporu:** bugünün toplamı, saat saat dağılım ve haftalık grafik. Yalnızca süre tutulur, hangi programları kullandığınız kaydedilmez.

### Genel
- Sistem tepsisinde çalışır; pencereyi kapatmak uygulamayı tepsiye indirir. İsteğe bağlı olarak Windows ile birlikte başlar.
- Açık / koyu / sistem teması, Türkçe ve İngilizce arayüz (çoklu dil altyapısı), 12/24 saat seçeneği.
- Hata durumlarında anlaşılır Türkçe mesajlar.

## Teknoloji

| Katman | Seçim |
|---|---|
| Çekirdek | Rust + [Tauri 2](https://tauri.app) |
| Arayüz | Svelte 5 + Tailwind CSS 4 + Vite |
| Yerel veri | SQLite (`rusqlite`) |
| Simgeler | [Lucide](https://lucide.dev) (SVG olarak projede, çevrimdışı çalışır) |

**Neden bu seçimler?** Tauri, Electron'a göre çok daha küçük kurulum paketi (~3 MB) ve düşük bellek kullanımı sunar; bu da tepside sürekli çalışan bir uygulama için önemlidir. Svelte, React'e göre daha az kodla daha küçük bir paket üretir ve tema/dil gibi durumları basit tutar.

Kullanılan başlıca Rust paketleri: `walkdir`, `blake3`, `trash`, `kamadak-exif`, `arboard`, `clipboard-master`, `rusqlite`, `chacha20poly1305`, `keyring`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`.

## Kurulum (kullanıcılar için)

Derlenmiş kurulum paketleri `src-tauri/target/release/bundle` klasöründe oluşur (aşağıya bakın):

- `nsis/Düzen-Baz_<sürüm>_x64-setup.exe` — kullanıcı bazında kurulur, yönetici izni gerekmez
- `msi/Düzen-Baz_<sürüm>_x64_tr-TR.msi`

> Paketler kod imzalı değildir; Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir.

## Geliştirme

Gereksinimler:

- [Rust](https://rustup.rs) (stable, MSVC araç zinciri)
- Node.js 20+
- Visual Studio Build Tools (C++ iş yükü) ve WebView2 (Windows 11'de hazır gelir)

```bash
npm install
npm run tauri dev      # geliştirme modunda başlat
```

Testler ve denetimler:

```bash
cd src-tauri && cargo test     # Rust birim testleri
npm run check                  # Svelte / TypeScript denetimi
```

Kurulum paketlerini üretmek için:

```bash
npm run tauri build
```

İlk derlemede paketleyici NSIS ve WiX araçlarını indirir (yalnızca derleme zamanında internet gerekir).

Geliştirme sırasında tarayıcı kısayolları (F12, Ctrl+Shift+I, sağ tık menüsü) bilerek kapalıdır. Hata ayıklamak için uygulamayı `DUZENBAZ_DEVTOOLS=1` ortam değişkeniyle başlatın; bu, Windows tarafındaki kilidi atlar.

## Proje yapısı

```
src/                     Svelte arayüzü
  modules/               organizer, renamer, clipboard, breaks, settings
  lib/                   ortak bileşenler, depolar, i18n (tr.json, en.json)
  assets/icons/          SVG simgeler (Lucide)
src-tauri/
  src/organizer/         tarama, sınıflandırma, kopya tespiti, geri alma
  src/renamer/           kurallar, EXIF, güvenli yeniden adlandırma
  src/clipboard/         dinleyici, şifreli depo, açılır pencere
  src/breaks/            durum makinesi, gece modu, ekran süresi
  src/db.rs              SQLite bağlantısı ve sürümlü şema geçişleri
  src/history.rs         geri alınabilir işlemlerin ortak günlüğü
```

Yeni bir dil eklemek için `src/lib/i18n/` altına bir JSON dosyası ekleyip `index.ts` içine kaydetmeniz yeterlidir.

## Güvenlik ve gizlilik

- Uygulama internete bağlanmaz; içerik güvenlik politikası (CSP) ağ erişimini kapalı tutar. Telemetri yoktur.
- **Pano verisi:** içerik ve etiketler SQLite'a yazılmadan önce ChaCha20-Poly1305 ile şifrelenir. Şifreleme anahtarı veritabanı dosyasında değil, işletim sisteminin güvenli deposunda (Windows Kimlik Bilgisi Yöneticisi) saklanır. Öğe türü, boyut ve zaman gibi üst veriler şifrelenmez.
- Bu, **tam dosya şifrelemesi (SQLCipher) değildir.** SQLCipher'ın Windows'ta OpenSSL derlemesi gerektirmesi nedeniyle alan düzeyinde şifreleme tercih edilmiştir.
- Dosya işlemlerinde hedef yollar arayüzden değil Rust tarafında yeniden hesaplanır; silme işlemleri yalnızca Geri Dönüşüm Kutusu'na yapılır.
- Uygulama beklenmedik şekilde çökerse gece modu ekran rengini sıcak tonda bırakabilir; uygulamayı yeniden açıp gece modunu kapatmanız yeterlidir.

## Platform durumu

| Özellik | Windows | macOS / Linux |
|---|---|---|
| Dosya Düzenleyici, Toplu Adlandırıcı | ✓ | derlenmesi beklenir, denenmedi |
| Pano geçmişi | ✓ | dinleme beklenir, parola yöneticisi işareti ve otomatik yapıştırma yok |
| Mola hatırlatıcı | ✓ | boşta kalma algısı yok (hep aktif sayılır) |
| Gece modu | ✓ | henüz yok |

## Lisans

[MIT](LICENSE) © 2026 BartuAbiHD

Simgeler [Lucide](https://lucide.dev) projesinden alınmıştır ve ISC lisansıyla dağıtılır; lisans metni `src/assets/icons/LICENSE-lucide.txt` dosyasındadır.
