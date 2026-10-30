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
| `duzenbaz-cli.exe` | Komut satırı aracı ve MCP sunucusu, kurulum gerekmez |
| `SHA256SUMS.txt` | Yukarıdaki dosyaların sağlama toplamları |

Gereksinim: Windows 10/11 (64 bit).

### Bilinen durumlar
- **Dosyalar kod imzalı değildir.** Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.
- Şimdilik yalnızca Windows. Gece modu ve otomatik yapıştırma macOS ve Linux'ta henüz çalışmaz.
- Bu bir ön sürümdür. Masaüstü uygulaması ve komut satırı aracı bir Windows 11 geliştirme makinesinde denendi; kurulum paketleri temiz bir makinede denenmedi.
- Pano verisi alan düzeyinde şifrelenir (tam dosya şifrelemesi değildir); ayrıntılar [README-TR](https://github.com/BartuAbiHD/Duzenbaz/blob/main/README-TR.md#güvenlik-ve-gizlilik) içindedir.
- Ortak çekirdeğin ürettiği bazı mesajlar (örneğin dosya hataları) komut satırı aracında da Türkçedir.

### Sağlama toplamları (SHA-256)
```
043fdc992feb99c967957ff5c90d969fa4de9443bf6268e73f0f1f7cddcaf6f7  Duzen-Baz_0.1.0_x64-setup.exe
b75b75aed8d5d8be5d85707fb54f8fa8533b1281f5644669f74933eb6d5c8429  Duzen-Baz_0.1.0_x64_tr-TR.msi
b3be133fe33fc96913e14f6d8f852283ceb38d8713f0c8cc9ff3197f0524bed2  duzenbaz-cli.exe
```
