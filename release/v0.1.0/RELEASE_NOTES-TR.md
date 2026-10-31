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
| `Duzen-Baz_0.1.0_amd64.deb` | Linux paketi (Debian/Ubuntu, x86_64; Xubuntu 22.04'te derlendi); `sudo apt install ./Duzen-Baz_0.1.0_amd64.deb` ile kurulur |
| `duzenbaz-cli_0.1.0_linux_x64.tar.gz` | Linux için komut satırı aracı ve MCP sunucusu (x86_64); açıp `./duzenbaz-cli` olarak çalıştırın |
| `duzenbaz-cli_0.1.0_macos_x64.tar.gz` | macOS için komut satırı aracı ve MCP sunucusu (Intel derlemesi); açıp `./duzenbaz-cli` olarak çalıştırın |
| `duzenbaz-cli.exe` | Komut satırı aracı ve MCP sunucusu, kurulum gerekmez |
| `SHA256SUMS.txt` | Yukarıdaki dosyaların sağlama toplamları |

Gereksinim: Windows 10/11 (64 bit) zip için macOS 11 ve üstü veya `.deb` için 64 bit Debian/Ubuntu tabanlı Linux.

### Bilinen durumlar
- **Dosyalar kod imzalı değildir.** Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.
- Windows ve macOS paketleri hazır; Linux paketleri x86_64 içindir (ARM'de denenmedi; orada kaynaktan derleyin). macOS uygulaması bir Mac'te derlendi ve noter onaylı değildir. Gece modu Linux'ta yalnızca X11 altında çalışır.
- Bu bir ön sürümdür. Masaüstü uygulaması ve komut satırı aracı bir Windows 11 geliştirme makinesinde denendi; kurulum paketleri temiz bir makinede denenmedi. Windows kurulum paketleri geliştirme makinesinde yerel olarak derlendi (Linux ve macOS paketleri kendi makinelerinde derlendi); hiçbir paket CI ile derlenmedi.
- Pano verisi alan düzeyinde şifrelenir (tam dosya şifrelemesi değildir); ayrıntılar [README-TR](https://github.com/BartuAbiHD/Duzenbaz/blob/main/README-TR.md#güvenlik-ve-gizlilik) içindedir.
- Ortak çekirdeğin ürettiği bazı mesajlar (örneğin dosya hataları) komut satırı aracında da Türkçedir.

### Sağlama toplamları (SHA-256)
```
783fe16284d3f8a2d5e9b435b06acf9b32f69c65fb0468a2534a7a97d165abba  Duzen-Baz_0.1.0_x64-setup.exe
c102adc7b8c5e8c061c883d8deec0dd19b46a165ec8a5ec25e5f0fe290ddc7dd  Duzen-Baz_0.1.0_x64_tr-TR.msi
760e6ffc4ac272959139566e00b7829496540854b45ca9c3ea704944f3138348  duzenbaz-cli.exe
efaaebff093d0549059f8786c13f2a5aae0e0ffd54d9648688b5adb37c2aa1b4  Duzen-Baz_0.1.0_macos_x64.zip
0c9a2118ef78d19094ac3513157d3508164e265f9734122512c3134023508af5  duzenbaz-cli_0.1.0_macos_x64.tar.gz
7abf07640f74c33d615844976106f849788cc97b19a7ceb6218d176a98f36753  Duzen-Baz_0.1.0_amd64.deb
b4098ab2336aee3c7c1bc71a4a0999356258e3fbd5052a9b6f697b70882ebce0  duzenbaz-cli_0.1.0_linux_x64.tar.gz
```
