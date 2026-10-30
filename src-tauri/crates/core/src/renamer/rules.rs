use super::exif::{created_date, exif_date, modified_date};
use crate::error::{AppError, AppResult};
use chrono::NaiveDateTime;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[default]
    None,
    Name,
    Modified,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaseMode {
    #[default]
    None,
    Lower,
    Upper,
    Title,
    Sentence,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[default]
    Prefix,
    Suffix,
    Replace,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DateSource {
    #[default]
    Exif,
    Modified,
    Created,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DateRule {
    pub enabled: bool,
    pub source: DateSource,
    pub format: String,
    pub position: Position,
    pub separator: String,
}

impl Default for DateRule {
    fn default() -> Self {
        Self { enabled: false, source: DateSource::Exif, format: "%Y-%m-%d".into(), position: Position::Prefix, separator: "_".into() }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NumRule {
    pub enabled: bool,
    pub start: i64,
    pub step: i64,
    pub digits: usize,
    pub position: Position,
    pub separator: String,
}

impl Default for NumRule {
    fn default() -> Self {
        Self { enabled: false, start: 1, step: 1, digits: 3, position: Position::Suffix, separator: "_".into() }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rules {
    pub sort: Sort,
    pub find: String,
    pub replace: String,
    pub case_sensitive: bool,
    pub case: CaseMode,
    pub date: DateRule,
    pub number: NumRule,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Unchanged,
    Conflict,
    Invalid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItem {
    pub src: String,
    pub old_name: String,
    pub new_name: String,
    pub status: Status,
    pub message: String,
}

fn apply_case(s: &str, mode: CaseMode) -> String {
    match mode {
        CaseMode::None => s.to_string(),
        CaseMode::Lower => s.to_lowercase(),
        CaseMode::Upper => s.to_uppercase(),
        CaseMode::Title => {
            let mut out = String::new();
            let mut word_start = true;
            for c in s.chars() {
                if c.is_alphanumeric() {
                    if word_start {
                        out.extend(c.to_uppercase());
                        word_start = false;
                    } else {
                        out.extend(c.to_lowercase());
                    }
                } else {
                    out.push(c);
                    word_start = c != '\'';
                }
            }
            out
        }
        CaseMode::Sentence => {
            let mut it = s.chars();
            match it.next() {
                None => String::new(),
                Some(f) => {
                    let mut o: String = f.to_uppercase().collect();
                    o.push_str(&it.as_str().to_lowercase());
                    o
                }
            }
        }
    }
}

fn place(stem: String, token: String, pos: Position, sep: &str) -> String {
    match pos {
        Position::Replace => token,
        _ if stem.is_empty() => token,
        Position::Prefix => format!("{token}{sep}{stem}"),
        Position::Suffix => format!("{stem}{sep}{token}"),
    }
}

fn format_date(dt: &NaiveDateTime, fmt: &str) -> AppResult<String> {
    let mut s = String::new();
    write!(s, "{}", dt.format(fmt)).map_err(|_| AppError::msg("Geçersiz tarih biçimi."))?;
    Ok(s)
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '_' } else { c })
        .collect();
    cleaned.trim_end_matches(['.', ' ']).trim_start().to_string()
}

const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3",
    "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Tek bir dosya için yeni adı üretir. İkinci değer, kullanıcıya gösterilecek isteğe bağlı nottur.
fn new_name(path: &Path, idx: usize, rules: &Rules) -> AppResult<(String, String)> {
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());
    let mut note = String::new();

    let mut s = stem;
    if !rules.find.is_empty() {
        let re = RegexBuilder::new(&regex::escape(&rules.find))
            .case_insensitive(!rules.case_sensitive)
            .build()
            .map_err(|_| AppError::msg("Aranan metin geçersiz."))?;
        s = re.replace_all(&s, regex::NoExpand(&rules.replace)).to_string();
    }
    s = apply_case(&s, rules.case);

    if rules.date.enabled {
        let primary = match rules.date.source {
            DateSource::Exif => exif_date(path),
            DateSource::Modified => modified_date(path),
            DateSource::Created => created_date(path),
        };
        let dt = match primary {
            Some(d) => d,
            None => {
                note = match rules.date.source {
                    DateSource::Exif => "EXIF tarihi yok; değiştirilme tarihi kullanıldı.".into(),
                    _ => "Seçilen tarih okunamadı; değiştirilme tarihi kullanıldı.".into(),
                };
                modified_date(path).ok_or_else(|| AppError::msg("Dosyanın tarihi okunamadı."))?
            }
        };
        s = place(s, format_date(&dt, &rules.date.format)?, rules.date.position, &rules.date.separator);
    }
    if rules.number.enabled {
        let n = rules.number.start.saturating_add(rules.number.step.saturating_mul(idx as i64));
        let token = format!("{n:0width$}", width = rules.number.digits.min(12));
        s = place(s, token, rules.number.position, &rules.number.separator);
    }
    s = format!("{}{}{}", rules.prefix, s, rules.suffix);

    let s = sanitize(&s);
    if s.is_empty() {
        return Err(AppError::msg("Ad boş kalıyor."));
    }
    if RESERVED.contains(&s.to_uppercase().as_str()) {
        return Err(AppError::msg("Bu ad Windows tarafından ayrılmış."));
    }
    let full = match ext {
        Some(e) => format!("{s}.{e}"),
        None => s,
    };
    if full.chars().count() > 255 {
        return Err(AppError::msg("Ad çok uzun."));
    }
    Ok((full, note))
}

fn key(p: &Path) -> String {
    p.to_string_lossy().to_lowercase()
}

pub fn preview(paths: &[String], rules: &Rules) -> AppResult<Vec<PreviewItem>> {
    let mut seen = HashSet::new();
    let mut files: Vec<PathBuf> =
        paths.iter().map(PathBuf::from).filter(|p| seen.insert(key(p))).collect();
    match rules.sort {
        Sort::None => {}
        Sort::Name => files.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase())),
        Sort::Modified => files.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok()),
    }
    let src_keys: HashSet<String> = files.iter().map(|p| key(p)).collect();

    let mut items = Vec::with_capacity(files.len());
    let mut targets: HashMap<String, usize> = HashMap::new();
    for (i, p) in files.iter().enumerate() {
        let old_name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let mut item = PreviewItem {
            src: p.to_string_lossy().to_string(),
            old_name: old_name.clone(),
            new_name: old_name.clone(),
            status: Status::Ok,
            message: String::new(),
        };
        if !p.is_file() {
            item.status = Status::Invalid;
            item.message = "Dosya bulunamadı.".into();
        } else {
            match new_name(p, i, rules) {
                Ok((n, note)) => {
                    item.status = if n == old_name { Status::Unchanged } else { Status::Ok };
                    item.new_name = n;
                    item.message = note;
                }
                Err(e) => {
                    item.status = Status::Invalid;
                    item.message = e.user_message();
                }
            }
            let target = p.with_file_name(&item.new_name);
            if item.status != Status::Invalid {
                *targets.entry(key(&target)).or_default() += 1;
            }
        }
        items.push(item);
    }

    for item in items.iter_mut().filter(|i| i.status == Status::Ok) {
        let target = Path::new(&item.src).with_file_name(&item.new_name);
        if targets.get(&key(&target)).copied().unwrap_or(0) > 1 {
            item.status = Status::Conflict;
            item.message = "Başka bir dosya da aynı ada dönüşüyor.".into();
        } else if target.exists() && !src_keys.contains(&key(&target)) {
            item.status = Status::Conflict;
            item.message = "Bu klasörde aynı adlı bir dosya zaten var.".into();
        }
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn run(paths: Vec<PathBuf>, rules: &Rules) -> Vec<PreviewItem> {
        let p: Vec<String> = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
        preview(&p, rules).unwrap()
    }

    #[test]
    fn find_replace_case_prefix_suffix_keep_extension() {
        let t = tempdir().unwrap();
        let f = t.path().join("Tatil FOTO.JPG");
        fs::write(&f, "x").unwrap();
        let rules = Rules {
            find: "foto".into(),
            replace: "resim".into(),
            case: CaseMode::Title,
            prefix: "2024-".into(),
            suffix: "-v2".into(),
            ..Default::default()
        };
        assert_eq!(run(vec![f], &rules)[0].new_name, "2024-Tatil Resim-v2.JPG");
    }

    #[test]
    fn numbering_follows_sort_order() {
        let t = tempdir().unwrap();
        let (b, a) = (t.path().join("b.txt"), t.path().join("a.txt"));
        fs::write(&a, "").unwrap();
        fs::write(&b, "").unwrap();
        let rules = Rules {
            sort: Sort::Name,
            number: NumRule { enabled: true, position: Position::Replace, digits: 2, start: 5, ..Default::default() },
            ..Default::default()
        };
        let items = run(vec![b, a], &rules);
        assert_eq!(items[0].old_name, "a.txt");
        assert_eq!(items[0].new_name, "05.txt");
        assert_eq!(items[1].new_name, "06.txt");
    }

    #[test]
    fn duplicate_targets_and_existing_files_are_conflicts() {
        let t = tempdir().unwrap();
        let (a, b, c) = (t.path().join("a.txt"), t.path().join("b.txt"), t.path().join("sabit.txt"));
        for f in [&a, &b, &c] {
            fs::write(f, "").unwrap();
        }
        let same = Rules { find: "a".into(), replace: "b".into(), ..Default::default() };
        let items = run(vec![a.clone(), b.clone()], &same);
        assert_eq!(items[0].status, Status::Conflict); // a→b ama b de kalıyor
        assert_eq!(items[1].status, Status::Unchanged);

        let onto_existing = Rules { find: "a".into(), replace: "sabit".into(), ..Default::default() };
        let items = run(vec![a], &onto_existing);
        assert_eq!(items[0].status, Status::Conflict);
    }

    #[test]
    fn invalid_characters_are_replaced_and_bad_date_format_is_reported() {
        let t = tempdir().unwrap();
        let f = t.path().join("x.txt");
        fs::write(&f, "").unwrap();
        let r = Rules { prefix: "a:b?".into(), ..Default::default() };
        assert_eq!(run(vec![f.clone()], &r)[0].new_name, "a_b_x.txt");

        let bad = Rules { date: DateRule { enabled: true, source: DateSource::Modified, format: "%Q".into(), ..Default::default() }, ..Default::default() };
        let items = run(vec![f], &bad);
        assert_eq!(items[0].status, Status::Invalid);
        assert!(items[0].message.contains("tarih biçimi"));
    }

    #[test]
    fn modified_date_is_added_as_prefix() {
        let t = tempdir().unwrap();
        let f = t.path().join("x.txt");
        fs::write(&f, "").unwrap();
        let r = Rules { date: DateRule { enabled: true, source: DateSource::Modified, format: "%Y".into(), ..Default::default() }, ..Default::default() };
        let n = &run(vec![f], &r)[0].new_name;
        assert!(n.len() == "2024_x.txt".len() && n.ends_with("_x.txt"), "{n}");
    }
}
