//! Dosya Düzenleyici'nin üst düzey işlemleri: arayüz, komut satırı ve MCP sunucusu bunları ortak kullanır.
use super::{
    dedupe::{self, DupGroup, TrashRequest},
    plan::{self, Failure, Options, PlannedMove},
};
use crate::{
    db::Db,
    error::{AppError, AppResult},
    history::{self, Batch},
};
use serde::Serialize;
use std::{collections::HashSet, path::PathBuf};

const MODULE: &str = "organizer";

pub fn check_root(root: &str) -> AppResult<PathBuf> {
    let p = PathBuf::from(root);
    if !p.is_dir() {
        return Err(AppError::msg("Seçilen klasör bulunamadı. Lütfen başka bir klasör seçin."));
    }
    if p.parent().is_none() {
        return Err(AppError::msg("Disk kök dizini düzenlenemez. Lütfen içindeki bir klasörü seçin."));
    }
    Ok(p)
}

/// Taşınacak dosyaların önizlemesi; hiçbir şeyi değiştirmez.
pub fn preview(root: &str, options: &Options) -> AppResult<Vec<PlannedMove>> {
    plan::build_plan(&check_root(root)?, options)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyReport {
    pub batch_id: String,
    pub moved: usize,
    pub failed: Vec<Failure>,
}

/// Planı yeniden hesaplar, yalnızca `selected` içindeki dosyaları taşır ve geri alınabilmesi için günlüğe yazar.
pub fn apply(db: &Db, root: &str, options: &Options, selected: &HashSet<String>) -> AppResult<ApplyReport> {
    let outcome = plan::apply_moves(&check_root(root)?, options, selected)?;
    let batch_id = history::new_batch_id();
    history::log(db, MODULE, "move", &batch_id, &outcome.done)?;
    Ok(ApplyReport { batch_id, moved: outcome.done.len(), failed: outcome.failed })
}

pub fn list_history(db: &Db) -> AppResult<Vec<Batch>> {
    history::list(db, MODULE)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub restored: usize,
    pub failed: Vec<Failure>,
}

pub fn undo(db: &Db, batch_id: &str) -> AppResult<UndoReport> {
    let rows = history::active(db, MODULE, batch_id)?;
    let pairs: Vec<(String, String)> = rows.iter().map(|(_, s, d)| (s.clone(), d.clone())).collect();
    let (ok, failed) = plan::undo_moves(&pairs);
    let ids: Vec<i64> = ok.iter().map(|i| rows[*i].0).collect();
    history::mark_undone(db, &ids)?;
    Ok(UndoReport { restored: ok.len(), failed })
}

pub fn find_duplicates(root: &str, recursive: bool) -> AppResult<Vec<DupGroup>> {
    dedupe::find_duplicates(&check_root(root)?, recursive)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashReport {
    pub trashed: usize,
    pub skipped: usize,
}

/// Kalıcı silme yoktur: dosyalar yalnızca Geri Dönüşüm Kutusu'na gönderilir. Her dosya,
/// saklanan dosyayla aynı içerikte olduğu yeniden doğrulanmadan gönderilmez.
pub fn trash_duplicates(requests: &[TrashRequest]) -> AppResult<TrashReport> {
    let mut trashed = 0;
    let mut skipped = 0;
    for req in requests {
        let verified = dedupe::verified_removals(req)?;
        skipped += req.remove.len() - verified.len();
        if verified.is_empty() {
            continue;
        }
        trash::delete_all(&verified).map_err(|_| {
            AppError::msg("Dosyalar Geri Dönüşüm Kutusu'na gönderilemedi. Dosyalar başka bir programda açık olabilir.")
        })?;
        trashed += verified.len();
    }
    Ok(TrashReport { trashed, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn apply_logs_and_undo_restores() {
        let db = Db::in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.jpg"), "x").unwrap();
        let root = dir.path().to_str().unwrap();
        let opts = Options { by_date: false };

        let plan = preview(root, &opts).unwrap();
        let selected: HashSet<String> = plan.iter().map(|m| m.src.clone()).collect();
        let report = apply(&db, root, &opts, &selected).unwrap();
        assert_eq!(report.moved, 1);
        assert!(dir.path().join("Resimler").join("a.jpg").exists());

        let hist = list_history(&db).unwrap();
        assert_eq!((hist.len(), hist[0].remaining), (1, 1));

        let undo = undo(&db, &report.batch_id).unwrap();
        assert_eq!(undo.restored, 1);
        assert!(dir.path().join("a.jpg").exists());
        assert_eq!(list_history(&db).unwrap()[0].remaining, 0);
    }

    #[test]
    fn rejects_missing_folder_and_drive_root() {
        assert!(check_root("Z:/olmayan/klasor").is_err());
        assert!(preview("Z:/olmayan/klasor", &Options { by_date: false }).is_err());
    }
}
