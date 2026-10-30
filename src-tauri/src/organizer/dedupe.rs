use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const PARTIAL_BYTES: u64 = 64 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DupFile {
    pub path: String,
    /// Değiştirilme zamanı (yerel saat); en eskisi "asıl" dosya sayılır.
    pub modified: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DupGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<DupFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashRequest {
    pub keep: String,
    pub remove: Vec<String>,
}

fn hash_file(path: &Path, limit: Option<u64>) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    match limit {
        Some(n) => hasher.update_reader(file.take(n))?,
        None => hasher.update_reader(file)?,
    };
    Ok(hasher.finalize().to_hex().to_string())
}

/// Boyut → ilk 64 KB özeti → tam blake3 özeti sırasıyla eler; çoğu dosya tam okunmaz.
pub fn find_duplicates(root: &Path, recursive: bool) -> AppResult<Vec<DupGroup>> {
    let mut walker = WalkDir::new(root).min_depth(1);
    if !recursive {
        walker = walker.max_depth(1);
    }
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for entry in walker.into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        if meta.len() > 0 {
            by_size.entry(meta.len()).or_default().push(entry.into_path());
        }
    }

    let mut groups = Vec::new();
    for (size, paths) in by_size.into_iter().filter(|(_, p)| p.len() > 1) {
        let mut partial: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for p in paths {
            if let Ok(h) = hash_file(&p, Some(PARTIAL_BYTES)) {
                partial.entry(h).or_default().push(p);
            }
        }
        for candidates in partial.into_values().filter(|c| c.len() > 1) {
            let mut full: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for p in candidates {
                if let Ok(h) = hash_file(&p, None) {
                    full.entry(h).or_default().push(p);
                }
            }
            for (hash, files) in full.into_iter().filter(|(_, f)| f.len() > 1) {
                let mut files: Vec<DupFile> = files
                    .into_iter()
                    .map(|p| {
                        let modified = std::fs::metadata(&p)
                            .and_then(|m| m.modified())
                            .map(|t| chrono::DateTime::<chrono::Local>::from(t).format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_default();
                        DupFile { path: p.to_string_lossy().to_string(), modified }
                    })
                    .collect();
                files.sort_by(|a, b| a.modified.cmp(&b.modified).then(a.path.cmp(&b.path)));
                groups.push(DupGroup { hash, size, files });
            }
        }
    }
    groups.sort_by_key(|g| std::cmp::Reverse(g.size * (g.files.len() as u64 - 1)));
    Ok(groups)
}

/// Silmeden önce her dosyanın "korunan" dosyayla gerçekten aynı içerikte olduğunu yeniden doğrular.
/// Döndürülen liste Geri Dönüşüm Kutusu'na gönderilecek güvenli yollardır.
pub fn verified_removals(req: &TrashRequest) -> AppResult<Vec<PathBuf>> {
    let keep = Path::new(&req.keep);
    if !keep.is_file() {
        return Err(AppError::msg("Saklanacak dosya bulunamadı; işlem güvenlik için durduruldu."));
    }
    let keep_hash = hash_file(keep, None)?;
    let mut ok = Vec::new();
    for r in &req.remove {
        let p = Path::new(r);
        if p == keep || !p.is_file() {
            continue;
        }
        if hash_file(p, None).map(|h| h == keep_hash).unwrap_or(false) {
            ok.push(p.to_path_buf());
        }
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_only_true_duplicates() {
        let tmp = tempdir().unwrap();
        let r = tmp.path();
        fs::write(r.join("a.txt"), "aynı içerik").unwrap();
        fs::write(r.join("b.txt"), "aynı içerik").unwrap();
        fs::write(r.join("c.txt"), "aynı içeriK").unwrap(); // aynı boyut, farklı içerik
        fs::write(r.join("bos1"), "").unwrap();
        fs::write(r.join("bos2"), "").unwrap();
        let g = find_duplicates(r, false).unwrap();
        assert_eq!(g.len(), 1, "boş dosyalar ve farklı içerikler sayılmamalı");
        assert_eq!(g[0].files.len(), 2);
    }

    #[test]
    fn removal_is_reverified() {
        let tmp = tempdir().unwrap();
        let r = tmp.path();
        fs::write(r.join("a.txt"), "1").unwrap();
        fs::write(r.join("b.txt"), "1").unwrap();
        fs::write(r.join("farkli.txt"), "2").unwrap();
        let req = TrashRequest {
            keep: r.join("a.txt").to_string_lossy().to_string(),
            remove: vec![
                r.join("b.txt").to_string_lossy().to_string(),
                r.join("farkli.txt").to_string_lossy().to_string(),
                r.join("a.txt").to_string_lossy().to_string(),
            ],
        };
        let ok = verified_removals(&req).unwrap();
        assert_eq!(ok, vec![r.join("b.txt")]);
    }
}
