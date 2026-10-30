use chrono::{DateTime, Local, NaiveDateTime};
use std::{fs::File, io::BufReader, path::Path};

pub fn parse_exif_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s.trim_end_matches('\0').trim(), "%Y:%m:%d %H:%M:%S").ok()
}

/// Fotoğrafın çekim tarihi (EXIF). Dosya EXIF içermiyorsa veya tarih geçersizse `None`.
pub fn exif_date(path: &Path) -> Option<NaiveDateTime> {
    let mut reader = BufReader::new(File::open(path).ok()?);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    [exif::Tag::DateTimeOriginal, exif::Tag::DateTimeDigitized, exif::Tag::DateTime]
        .into_iter()
        .find_map(|tag| {
            let field = exif.get_field(tag, exif::In::PRIMARY)?;
            match &field.value {
                exif::Value::Ascii(v) => parse_exif_datetime(&String::from_utf8_lossy(v.first()?)),
                _ => None,
            }
        })
}

pub fn modified_date(path: &Path) -> Option<NaiveDateTime> {
    let t = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Local>::from(t).naive_local())
}

pub fn created_date(path: &Path) -> Option<NaiveDateTime> {
    let t = std::fs::metadata(path).ok()?.created().ok()?;
    Some(DateTime::<Local>::from(t).naive_local())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exif_timestamps() {
        let d = parse_exif_datetime("2023:07:14 18:30:05\0").unwrap();
        assert_eq!(d.format("%Y-%m-%d %H:%M:%S").to_string(), "2023-07-14 18:30:05");
        assert!(parse_exif_datetime("0000:00:00 00:00:00").is_none());
        assert!(parse_exif_datetime("bozuk").is_none());
    }
}
