# Kod imzalama (SignPath Foundation)

Windows kurulum paketlerini [SignPath Foundation](https://signpath.org) üzerinden ücretsiz imzalamak için adımlar. Hizmetin koşulları ve alan adları zamanla değişebilir; her adımda güncel [SignPath belgelerine](https://about.signpath.io/documentation/) bakın.

## 1. Başvuru (sizin yapmanız gerekir)

1. https://signpath.org/ adresinden açık kaynak projeler için başvuru formunu doldurun.
2. Projenin SignPath'in koşullarına uyması gerekir: OSI onaylı lisans (bu proje MIT), herkese açık depo, etkin bakım, kötü amaçlı yazılım içermemesi.
3. Başvuru onaylanınca SignPath size bir **organizasyon** ve bir **proje** açar. Şu değerleri not alın:
   - Organization ID
   - Proje kısa adı (project slug)
   - İmza ilkesi kısa adı (signing policy slug, örn. `release-signing`)
   - Yapıt yapılandırması kısa adı (artifact configuration slug)
4. SignPath panelinde bir **API belirteci (token)** oluşturun.
5. SignPath panelinde yapıt yapılandırmasını `.signpath/artifact-configuration.xml` dosyasındaki içerikle oluşturun.

## 2. GitHub ayarları

Depo > Settings > Secrets and variables > Actions:

| Tür | Ad | Değer |
|---|---|---|
| Secret | `SIGNPATH_API_TOKEN` | SignPath API belirteci |
| Variable | `SIGNPATH_ORGANIZATION_ID` | Organization ID |
| Variable | `SIGNPATH_PROJECT_SLUG` | proje kısa adı |
| Variable | `SIGNPATH_SIGNING_POLICY_SLUG` | imza ilkesi kısa adı |
| Variable | `SIGNPATH_ARTIFACT_CONFIGURATION_SLUG` | yapıt yapılandırması kısa adı |

`SIGNPATH_ORGANIZATION_ID` tanımlanmadıkça iş akışı imza adımını atlar ve imzasız paket yayımlar.

## 3. README'ye eklenecek metin (yalnızca onaydan sonra)

SignPath Foundation, imza veren projelerin kod imzalama ilkesini ve katkıcı rollerini duyurmasını ister. Onay çıktığında README'ye şu bölüm eklenmelidir (güncel gerekli metin için SignPath'in başvuru e-postasına bakın):

> Ücretsiz kod imzalama [SignPath.io](https://about.signpath.io) tarafından sağlanır, sertifika [SignPath Foundation](https://signpath.org) tarafından verilir.
>
> **Gizlilik:** Uygulama kullanıcı verisini hiçbir yere göndermez; bkz. [Güvenlik ve gizlilik](../README.md#güvenlik-ve-gizlilik).

Onay gelmeden bu metni eklemeyin: henüz doğru değildir.

## 4. Sürüm çıkarmak

```bash
# package.json ve src-tauri/tauri.conf.json içindeki sürümü güncelleyin, commit edin, sonra:
git tag -a v0.2.0 -m "Düzen-Baz 0.2.0"
git push origin main v0.2.0
```

Etiket gönderilince `.github/workflows/release.yml` derler, imzalar ve Release'e yükler.

## Bilmeniz gerekenler

- **SmartScreen:** İmza, "bilinmeyen yayıncı" yerine yayıncı adınızı gösterir. Ancak SmartScreen uyarısının tamamen kalkması için indirme itibarının zamanla oluşması gerekir; ilk sürümlerde uyarı yine görülebilir.
- **Kapsam:** Yalnızca kurulum dosyaları (`.exe` ve `.msi`) imzalanır. Kurulumdan sonra yüklenen `duzenbaz.exe` bu akışla imzalanmaz.
- **Sürüm eşitliği:** `package.json` ve `tauri.conf.json` sürümleri aynı olmalı; iş akışı dosya adını `package.json`'dan alır.
- **Doğrulanmadı:** Bu iş akışını ve XML yapılandırmasını çalıştırıp denemedim; SignPath hesabı olmadan denenemez. İlk çalıştırmada küçük düzeltmeler gerekebilir.
