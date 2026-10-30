use crate::error::{AppError, AppResult};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

fn key(p: &Path) -> String {
    super::path_key(p)
}

fn name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

/// Tüm çiftleri ya birlikte yeniden adlandırır ya da hiçbirini değiştirmez.
/// İki aşamalıdır (önce geçici ad, sonra hedef); böylece a↔b takası veya a→b, b→c zincirleri güvenle çalışır.
pub fn rename_all(pairs: &[(PathBuf, PathBuf)]) -> AppResult<()> {
    let srcs: HashSet<String> = pairs.iter().map(|(s, _)| key(s)).collect();
    let mut dsts = HashSet::new();
    for (s, d) in pairs {
        if !s.is_file() {
            return Err(AppError::msg(format!("Dosya bulunamadı: {}. Hiçbir dosya değiştirilmedi.", name(s))));
        }
        if !dsts.insert(key(d)) {
            return Err(AppError::msg(format!("Birden fazla dosya \"{}\" adını alacaktı. Hiçbir dosya değiştirilmedi.", name(d))));
        }
        if d.exists() && !srcs.contains(&key(d)) {
            return Err(AppError::msg(format!("\"{}\" adı zaten kullanılıyor. Hiçbir dosya değiştirilmedi.", name(d))));
        }
    }

    let tmps: Vec<PathBuf> = pairs
        .iter()
        .enumerate()
        .map(|(i, (s, _))| s.with_file_name(format!(".duzenbaz-{}-{i}.tmp", std::process::id())))
        .collect();

    for (i, (s, _)) in pairs.iter().enumerate() {
        if let Err(e) = fs::rename(s, &tmps[i]) {
            for j in 0..i {
                let _ = fs::rename(&tmps[j], &pairs[j].0);
            }
            return Err(e.into());
        }
    }
    for (i, (_, d)) in pairs.iter().enumerate() {
        if let Err(e) = fs::rename(&tmps[i], d) {
            for j in 0..i {
                let _ = fs::rename(&pairs[j].1, &tmps[j]);
            }
            for j in 0..pairs.len() {
                let _ = fs::rename(&tmps[j], &pairs[j].0);
            }
            return Err(e.into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn swaps_and_chains_work() {
        let t = tempdir().unwrap();
        let r = t.path();
        fs::write(r.join("a"), "A").unwrap();
        fs::write(r.join("b"), "B").unwrap();
        rename_all(&[(r.join("a"), r.join("b")), (r.join("b"), r.join("a"))]).unwrap();
        assert_eq!(fs::read_to_string(r.join("a")).unwrap(), "B");
        assert_eq!(fs::read_to_string(r.join("b")).unwrap(), "A");
    }

    #[test]
    fn collision_changes_nothing() {
        let t = tempdir().unwrap();
        let r = t.path();
        fs::write(r.join("a"), "A").unwrap();
        fs::write(r.join("b"), "B").unwrap();
        fs::write(r.join("c"), "C").unwrap();
        let err = rename_all(&[(r.join("a"), r.join("x")), (r.join("b"), r.join("c"))]);
        assert!(err.is_err());
        assert_eq!(fs::read_to_string(r.join("a")).unwrap(), "A");
        assert!(!r.join("x").exists());
        assert_eq!(fs::read_to_string(r.join("c")).unwrap(), "C");
    }

    #[test]
    fn case_only_rename() {
        let t = tempdir().unwrap();
        let r = t.path();
        fs::write(r.join("Foto.JPG"), "1").unwrap();
        rename_all(&[(r.join("Foto.JPG"), r.join("foto.jpg"))]).unwrap();
        let names: Vec<String> = fs::read_dir(r).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
        assert_eq!(names, vec!["foto.jpg"]);
    }
}
