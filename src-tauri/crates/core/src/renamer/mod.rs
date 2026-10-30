pub mod exif;
pub mod fsops;
pub mod rules;
pub mod service;

/// İki yolun aynı dosyayı gösterip göstermediğini anlamak için karşılaştırma anahtarı.
/// Windows ve macOS'ta (varsayılan) dosya sistemi büyük/küçük harfe duyarsızdır, bu yüzden `A.txt` ile `a.txt`
/// aynı dosyadır; Linux'ta ise iki ayrı dosyadır ve yanlış çakışma uyarısı verilmemelidir.
pub(crate) fn path_key(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    if cfg!(any(windows, target_os = "macos")) {
        s.to_lowercase()
    } else {
        s.into_owned()
    }
}
