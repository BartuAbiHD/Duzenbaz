## Düzen-Baz 0.1.0 — ilk sürüm

**Diller:** [English](RELEASE_NOTES.md) · Türkçe

Dosyalarınızı, panonuzu ve göz sağlığınızı bir arada yöneten hafif bir masaüstü uygulaması. Tüm veriler bilgisayarınızda kalır; internet bağlantısı gerekmez, telemetri yoktur. Ayrıca bir **MCP sunucusu** olarak da çalışan bir komut satırı aracı içerir; böylece Claude gibi yapay zekâ asistanları aynı özellikleri kullanabilir.

### Modüller
- **Dosya Düzenleyici:** klasörü türe ve tarihe göre ayırır, kopya dosyaları bulur (Geri Dönüşüm Kutusu'na gönderir, kalıcı silme yok); ön izleme ve geri alma
- **Toplu Adlandırıcı:** sürükle-bırak, EXIF dahil tarih, sıra numarası, önek/sonek, bul-değiştir, canlı ön izleme ve geri alma
- **Pano Geçmişi:** şifreli yerel geçmiş, `Ctrl+Shift+V` ile açılan aranabilir pencere, sabitleme, kategori ve etiket, parola yöneticisi içeriğini yoksayma
- **Mola ve Göz Sağlığı:** mola hatırlatıcı, gece modu, günlük ekran süresi raporu

### Yeni: `duzenbaz-cli` ve MCP sunucusu
`duzenbaz-cli.exe`, masaüstü uygulamasıyla aynı çekirdeği ve aynı yerel veriyi kullanan, kurulum gerektirmeyen tek bir dosyadır.

- Komutlar: `organize`, `duplicates`, `rename`, `clipboard`, `screen-time` (makine okunur çıktı için `--json` ekleyin)
- `duzenbaz-cli mcp`, stdio üzerinden 11 araçlı bir [Model Context Protocol](https://modelcontextprotocol.io) sunucusu çalıştırır (pano araması 12. araçtır, `--allow-clipboard` vermedikçe kapalıdır)
- Varsayılanı güvenlidir: araçlar yalnızca izin verilen klasörlerde çalışır (varsayılan: İndirilenler, `--allow-dir` ile değişir), `--read-only` bir şeyi değiştiren tüm araçları gizler, dosyalar yalnızca Geri Dönüşüm Kutusu'na gider ve her değişiklik geri alınabilir

```bash
claude mcp add duzenbaz -- "C:\yol\duzenbaz-cli.exe" mcp --allow-dir "C:\Users\ben\Downloads"
```

Tüm komutlar ve Claude Desktop yapılandırması için [README-TR](https://github.com/BartuAbiHD/Duzenbaz/blob/main/README-TR.md#komut-satırı-aracı-ve-mcp-sunucusu) dosyasına bakın.

### İndirilenler
| Dosya | Nedir |
|---|---|
| `Duzen-Baz_0.1.0_x64-setup.exe` | Önerilen kurulum; kullanıcı bazında kurulur, yönetici izni gerekmez |
| `Duzen-Baz_0.1.0_x64_tr-TR.msi` | MSI paketi |
| `Duzen-Baz_0.1.0_macos_x64.zip` | macOS uygulaması (Intel derlemesi, Apple Silicon'da Rosetta ile çalışır); zip'i açıp Uygulamalar'a sürükleyin. İmzasızdır: ilk açılışta sağ tıklayıp Aç'ı seçin |
| `duzenbaz-cli.exe` | Komut satırı aracı ve MCP sunucusu, kurulum gerekmez |
| `SHA256SUMS.txt` | Yukarıdaki dosyaların sağlama toplamları |

Gereksinim: Windows 10/11 (64 bit) veya zip için macOS 11 ve üstü.

### Bilinen durumlar
- **Dosyalar kod imzalı değildir.** Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.
- Windows ve macOS paketleri hazır; Linux için kaynaktan derleyin. macOS uygulaması bir Mac'te derlendi ve noter onaylı değildir. Gece modu Linux'ta yalnızca X11 altında çalışır.
- Bu bir ön sürümdür. Masaüstü uygulaması ve komut satırı aracı bir Windows 11 geliştirme makinesinde denendi; kurulum paketleri temiz bir makinede denenmedi. Dosyalar, tüm testler geçtikten sonra bu deponun GitHub Actions iş akışıyla temiz bir Windows makinesinde derlendi.
- Pano verisi alan düzeyinde şifrelenir (tam dosya şifrelemesi değildir); ayrıntılar [README-TR](https://github.com/BartuAbiHD/Duzenbaz/blob/main/README-TR.md#güvenlik-ve-gizlilik) içindedir.
- Ortak çekirdeğin ürettiği bazı mesajlar (örneğin dosya hataları) komut satırı aracında da Türkçedir.

### Sağlama toplamları (SHA-256)
```
e3bad363e834b7f888a71b3486bcdde78fb205acc8c7aa45fdbda54ab0cc0f51  Duzen-Baz_0.1.0_x64-setup.exe
dba70ae496a249e030a6ffb6da3da24a5086f53ac3f53d02060eb6ee800774d2  Duzen-Baz_0.1.0_x64_tr-TR.msi
eb420f1054ebe6f1791720858950b01eb84f16df9902160fa8e8ae26d24e45fb  duzenbaz-cli.exe
efaaebff093d0549059f8786c13f2a5aae0e0ffd54d9648688b5adb37c2aa1b4  Duzen-Baz_0.1.0_macos_x64.zip
```
