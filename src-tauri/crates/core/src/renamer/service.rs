//! Toplu Adlandırıcı'nın üst düzey işlemleri: arayüz, komut satırı ve MCP sunucusu bunları ortak kullanır.
use super::{
    fsops,
    rules::{self, PreviewItem, Rules, Status},
};
use crate::{
    db::Db,
    error::{AppError, AppResult},
    history::{self, Batch},
};
use serde::Serialize;
use std::{collections::HashSet, fs, path::PathBuf};

const MODULE: &str = "renamer";

/// Verilen yolları dosya listesine çevirir: klasörün doğrudan içindeki dosyalar eklenir, gizli dosyalar atlanır.
pub fn expand(paths: &[String]) -> AppResult<Vec<String>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths.iter().map(PathBuf::from) {
        let candidates: Vec<PathBuf> = if p.is_dir() {
            let mut v: Vec<PathBuf> = fs::read_dir(&p)?.filter_map(Result::ok).map(|e| e.path()).collect();
            v.sort();
            v
        } else {
            vec![p]
        };
        for c in candidates {
            let name = c.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if c.is_file() && !name.starts_with('.') && seen.insert(super::path_key(&c)) {
                out.push(c.to_string_lossy().to_string());
            }
        }
    }
    Ok(out)
}

/// Yeni adların önizlemesi; hiçbir şeyi değiştirmez.
pub fn preview(paths: &[String], rules: &Rules) -> AppResult<Vec<PreviewItem>> {
    rules::preview(paths, rules)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameReport {
    pub batch_id: String,
    pub renamed: usize,
}

/// Ön izlemeyi yeniden hesaplar; çakışan veya geçersiz ad varsa hiçbir dosyayı değiştirmez.
/// Başarılı olursa işlem geri alınabilsin diye günlüğe yazılır.
pub fn apply(db: &Db, paths: &[String], rules: &Rules) -> AppResult<RenameReport> {
    let items = rules::preview(paths, rules)?;
    if items.iter().any(|i| matches!(i.status, Status::Conflict | Status::Invalid)) {
        return Err(AppError::msg("Çakışan veya geçersiz adlar var. Önce bunları düzeltin; hiçbir dosya değiştirilmedi."));
    }
    let pairs: Vec<(PathBuf, PathBuf)> = items
        .iter()
        .filter(|i| i.status == Status::Ok)
        .map(|i| (PathBuf::from(&i.src), PathBuf::from(&i.src).with_file_name(&i.new_name)))
        .collect();
    fsops::rename_all(&pairs)?;

    let batch_id = history::new_batch_id();
    let logged: Vec<(String, String)> =
        pairs.iter().map(|(s, d)| (s.to_string_lossy().to_string(), d.to_string_lossy().to_string())).collect();
    history::log(db, MODULE, "rename", &batch_id, &logged)?;
    Ok(RenameReport { batch_id, renamed: pairs.len() })
}

pub fn list_history(db: &Db) -> AppResult<Vec<Batch>> {
    history::list(db, MODULE)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub restored: usize,
}

pub fn undo(db: &Db, batch_id: &str) -> AppResult<UndoReport> {
    let rows = history::active(db, MODULE, batch_id)?;
    // Geri alırken yönler tersine çevrilir: yeni ad → eski ad.
    let pairs: Vec<(PathBuf, PathBuf)> = rows.iter().map(|(_, old, new)| (PathBuf::from(new), PathBuf::from(old))).collect();
    fsops::rename_all(&pairs)?;
    let ids: Vec<i64> = rows.iter().map(|r| r.0).collect();
    history::mark_undone(db, &ids)?;
    Ok(UndoReport { restored: ids.len() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_conflict_is_refused_and_undo_roundtrips() {
        let db = Db::in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        for n in ["a.txt", "b.txt"] {
            fs::write(dir.path().join(n), n).unwrap();
        }
        let files = expand(&[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!(files.len(), 2);

        // Her ikisini de aynı ada çevirmeye çalışan kural çakışma üretir.
        let conflict = Rules { find: "a".into(), replace: "b".into(), ..Default::default() };
        let items = preview(&files, &conflict).unwrap();
        assert!(items.iter().any(|i| i.status == Status::Conflict));
        assert!(apply(&db, &files, &conflict).is_err());
        assert!(dir.path().join("a.txt").exists(), "çakışmada hiçbir dosya değişmemeli");

        let rules = Rules { prefix: "yeni_".into(), ..Default::default() };
        let report = apply(&db, &files, &rules).unwrap();
        assert_eq!(report.renamed, 2);
        assert!(dir.path().join("yeni_a.txt").exists());

        assert_eq!(undo(&db, &report.batch_id).unwrap().restored, 2);
        assert!(dir.path().join("a.txt").exists() && !dir.path().join("yeni_a.txt").exists());
    }
}
