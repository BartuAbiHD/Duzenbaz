use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Images,
    Documents,
    Videos,
    Music,
    Archives,
    Programs,
    Other,
}

impl Category {
    /// Hedef alt klasörün adı (kullanıcı bunu dosya gezgininde görür).
    pub fn folder(self) -> &'static str {
        match self {
            Category::Images => "Resimler",
            Category::Documents => "Belgeler",
            Category::Videos => "Videolar",
            Category::Music => "Müzik",
            Category::Archives => "Arşivler",
            Category::Programs => "Programlar",
            Category::Other => "Diğer",
        }
    }
}

pub fn classify(path: &Path) -> Category {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "heic" | "tif" | "tiff" | "ico" | "raw" | "cr2"
        | "nef" | "arw" | "avif" => Category::Images,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt" | "ods" | "odp" | "csv"
        | "md" | "epub" | "mobi" | "pages" | "numbers" | "key" => Category::Documents,
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "3gp" => Category::Videos,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "aiff" | "mid" | "midi" => Category::Music,
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "tgz" | "zst" => Category::Archives,
        "exe" | "msi" | "msix" | "apk" | "dmg" | "pkg" | "deb" | "rpm" | "appimage" | "bat" | "cmd" | "ps1" | "jar" => {
            Category::Programs
        }
        _ => Category::Other,
    }
}

/// Dokunulmaması gereken dosyalar: gizli/sistem dosyaları ve yarım kalmış indirmeler.
pub fn is_skippable(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with('.')
        || lower.starts_with("~$")
        || lower == "desktop.ini"
        || lower == "thumbs.db"
        || [".crdownload", ".part", ".partial", ".tmp", ".download", ".opdownload"]
            .iter()
            .any(|s| lower.ends_with(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_extension_case_insensitively() {
        assert_eq!(classify(Path::new("a.JPG")), Category::Images);
        assert_eq!(classify(Path::new("a.pdf")), Category::Documents);
        assert_eq!(classify(Path::new("setup.exe")), Category::Programs);
        assert_eq!(classify(Path::new("noext")), Category::Other);
    }

    #[test]
    fn skips_partial_downloads_and_hidden() {
        assert!(is_skippable("film.mkv.crdownload"));
        assert!(is_skippable(".hidden"));
        assert!(is_skippable("desktop.ini"));
        assert!(!is_skippable("rapor.pdf"));
    }
}
