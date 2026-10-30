use super::classify::{classify, is_skippable};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// true ise türün altında ayrıca "YYYY-AA" klasörleri oluşturulur.
    pub by_date: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedMove {
    pub src: String,
    pub dst: String,
    pub category: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub done: Vec<(String, String)>,
    pub failed: Vec<Failure>,
}

fn unique_path(dir: &Path, file_name: &str, taken: &HashSet<PathBuf>) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() && !taken.contains(&candidate) {
        return candidate;
    }
    let p = Path::new(file_name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let ext = p.extension().and_then(|s| s.to_str());
    (1..)
        .map(|n| match ext {
            Some(e) => dir.join(format!("{stem} ({n}).{e}")),
            None => dir.join(format!("{stem} ({n})")),
        })
        .find(|c| !c.exists() && !taken.contains(c))
        .expect("sonsuz dizi")
}

/// Seçilen klasörün yalnızca doğrudan içindeki dosyaları ele alınır; alt klasörlere dokunulmaz.
pub fn build_plan(root: &Path, opts: &Options) -> AppResult<Vec<PlannedMove>> {
    let mut taken = HashSet::new();
    let mut plan = Vec::new();
    for entry in WalkDir::new(root).min_depth(1).max_depth(1).sort_by_file_name() {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if is_skippable(&name) {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let mut dir = root.join(classify(entry.path()).folder());
        if opts.by_date {
            if let Ok(modified) = meta.modified() {
                let dt: DateTime<Local> = modified.into();
                dir = dir.join(dt.format("%Y-%m").to_string());
            }
        }
        let dst = unique_path(&dir, &name, &taken);
        taken.insert(dst.clone());
        plan.push(PlannedMove {
            src: entry.path().to_string_lossy().to_string(),
            dst: dst.to_string_lossy().to_string(),
            category: classify(entry.path()).folder().to_string(),
            size: meta.len(),
        });
    }
    Ok(plan)
}

/// Planı yeniden hesaplar ve yalnızca `selected` içindeki kaynakları taşır.
/// Hedef yolu arayüzden alınmaz; böylece arayüz kök klasörün dışına yazdıramaz.
pub fn apply_moves(root: &Path, opts: &Options, selected: &HashSet<String>) -> AppResult<Outcome> {
    let mut out = Outcome::default();
    for m in build_plan(root, opts)?.into_iter().filter(|m| selected.contains(&m.src)) {
        let dst = Path::new(&m.dst);
        let result = (|| -> AppResult<()> {
            if dst.exists() {
                return Err(AppError::msg("Hedefte aynı adlı bir dosya zaten var."));
            }
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&m.src, dst)?;
            Ok(())
        })();
        match result {
            Ok(()) => out.done.push((m.src, m.dst)),
            Err(e) => out.failed.push(Failure { path: m.src, message: e.user_message() }),
        }
    }
    Ok(out)
}

/// (kaynak, hedef) çiftlerini ters sırada eski yerine taşır. Başarılı olanların indeksini döndürür.
pub fn undo_moves(ops: &[(String, String)]) -> (Vec<usize>, Vec<Failure>) {
    let mut ok = Vec::new();
    let mut failed = Vec::new();
    for (i, (src, dst)) in ops.iter().enumerate().rev() {
        let (src_p, dst_p) = (Path::new(src), Path::new(dst));
        let result = (|| -> AppResult<()> {
            if !dst_p.exists() {
                return Err(AppError::msg("Dosya taşındığı yerde bulunamadı; silinmiş veya yeniden adlandırılmış olabilir."));
            }
            if src_p.exists() {
                return Err(AppError::msg("Eski konumda aynı adlı başka bir dosya var."));
            }
            if let Some(parent) = src_p.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(dst_p, src_p)?;
            // Düzenleme sırasında açılan boş tür ve tarih klasörlerini temizle.
            let mut dir = dst_p.parent();
            for _ in 0..2 {
                match dir {
                    Some(d) if Some(d) != src_p.parent() && fs::remove_dir(d).is_ok() => dir = d.parent(),
                    _ => break,
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => ok.push(i),
            Err(e) => failed.push(Failure { path: src.clone(), message: e.user_message() }),
        }
    }
    (ok, failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn touch(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(name), body).unwrap();
    }

    #[test]
    fn plan_apply_and_undo_roundtrip() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        touch(root, "a.jpg", "x");
        touch(root, "b.pdf", "y");
        touch(root, "c.mkv.crdownload", "z");
        fs::create_dir(root.join("alt")).unwrap();
        let opts = Options { by_date: true };

        let plan = build_plan(root, &opts).unwrap();
        assert_eq!(plan.len(), 2);
        assert!(plan.iter().any(|m| m.category == "Resimler"));

        let selected: HashSet<String> = plan.iter().map(|m| m.src.clone()).collect();
        let out = apply_moves(root, &opts, &selected).unwrap();
        assert_eq!(out.done.len(), 2);
        assert!(!root.join("a.jpg").exists());
        assert!(root.join("c.mkv.crdownload").exists());

        let (ok, failed) = undo_moves(&out.done);
        assert_eq!(ok.len(), 2);
        assert!(failed.is_empty());
        assert!(root.join("a.jpg").exists());
        assert!(!root.join("Resimler").exists(), "boş klasörler temizlenmeli");
    }

    #[test]
    fn name_conflicts_get_a_suffix_and_nothing_is_overwritten() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("Resimler")).unwrap();
        touch(&root.join("Resimler"), "a.jpg", "eski");
        touch(root, "a.jpg", "yeni");
        let opts = Options { by_date: false };
        let plan = build_plan(root, &opts).unwrap();
        assert!(plan[0].dst.ends_with("a (1).jpg"));
        let selected = HashSet::from([plan[0].src.clone()]);
        apply_moves(root, &opts, &selected).unwrap();
        assert_eq!(fs::read_to_string(root.join("Resimler").join("a.jpg")).unwrap(), "eski");
    }

    #[test]
    fn undo_refuses_to_overwrite() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        touch(root, "k.txt", "1");
        touch(root, "k2.txt", "2");
        let ops = vec![(root.join("k.txt").to_string_lossy().to_string(), root.join("k2.txt").to_string_lossy().to_string())];
        let (ok, failed) = undo_moves(&ops);
        assert!(ok.is_empty());
        assert_eq!(failed.len(), 1);
        assert_eq!(fs::read_to_string(root.join("k.txt")).unwrap(), "1");
    }
}
