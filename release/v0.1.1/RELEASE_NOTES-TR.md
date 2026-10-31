## Duzen-Baz 0.1.1

**Diller:** [English](RELEASE_NOTES.md) · Türkçe

### Yeni
- **Güncelleme kontrolü (Ayarlar → Güncellemeler).** Varsayılan olarak kapalıdır. Açarsanız uygulama açılırken GitHub'dan en son sürüm numarasını sorar ve onayınızla yeni sürümü indirip kurabilir. Her güncelleme imzayla doğrulanır. Kapalıyken uygulama internete hiç bağlanmaz. Linux'ta (`.deb`) uygulama yalnızca yeni sürümü bildirir ve indirme sayfasını açar.

### Düzeltilenler
- **Mola hatırlatıcısı hiç gelmiyordu.** 20 dakikalık sayaç, örneğin okurken, bir dakikadan uzun süre klavye veya fareye dokunulmayınca sıfırlanıyordu. Artık sayaç yalnızca duruyor; bilgisayardan 5 dakikadan uzun ayrılınca sıfırlanıyor.
- **Linux: sanal makinelerde pencere boş kalıyor veya tıklamalar çalışmıyordu.** Web görünümü için yazılım çizimi artık varsayılan (`WEBKIT_DISABLE_COMPOSITING_MODE=1`; kapatmak için `0` verin).
- **Linux: pano anahtarı açılışı kilitleyebiliyordu** (sistem parola deposu parola sorarken); artık arka planda yükleniyor.
- **Linux: pano geçmişinden yapıştırma.** Kopyalanan öğe, hedef program alana kadar kullanılabilir tutuluyor ve `Ctrl+V` gönderilmeden önce odak önceki pencereye geri veriliyor. Sisteminizde yapıştırma yine olmuyorsa uygulamayı `DUZENBAZ_DEBUG=1` ile başlatıp `pano:` satırlarıyla bir kayıt açın.
- Windows kurulumu artık `Düzen-Baz` yerine `Duzen-Baz` klasörüne kurulur.

### İndirilenler
| Dosya | Nedir |
|---|---|
| `Duzen-Baz_0.1.1_x64-setup.exe` | Önerilen kurulum; kullanıcı bazında kurulur, yönetici izni gerekmez |
| `Duzen-Baz_0.1.1_x64_tr-TR.msi` | MSI paketi |
| `duzenbaz-cli.exe` | Komut satırı aracı ve MCP sunucusu, kurulum gerekmez |
| `latest.json` | Uygulamanın okuduğu güncelleme bilgisi (indirmeniz gerekmez) |
| `SHA256SUMS.txt` | Yukarıdaki dosyaların sağlama toplamları |

Gereksinim: Windows 10/11 (64 bit). Bu sürümün macOS ve Linux paketleri ayrıca eklenir.

### Bilinen durumlar
- **Dosyalar kod imzalı değildir** (güncelleme imzası, Windows kod imzalamadan ayrıdır). Windows SmartScreen "bilinmeyen yayıncı" uyarısı gösterebilir. İndirdiğiniz dosyayı `SHA256SUMS.txt` ile doğrulayabilirsiniz.
- 0.1.0 sürümünde güncelleme kontrolü yoktur; 0.1.1'i bir kez elle kurun, sonraki sürümler kendini güncelleyebilir.

### Sağlama toplamları (SHA-256)
```
a129a9c61c39e928c2658643512528c4b758e88e1999535b87485a14b2b2d8aa  Duzen-Baz_0.1.1_x64-setup.exe
621763d51a217846695a10065f37e757ffd9b756477937681fd7d4c28cb90218  Duzen-Baz_0.1.1_x64_tr-TR.msi
39413522ac955ccd52c726630c93a9797600baed7960c478837f25df230fd228  duzenbaz-cli.exe
```
