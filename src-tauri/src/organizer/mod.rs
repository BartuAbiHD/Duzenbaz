mod classify;
mod dedupe;
mod plan;

use crate::{
    db::Db,
    error::{AppError, AppResult},
    history::{self, Batch},
    util::blocking,
};
use dedupe::{DupGroup, TrashRequest};
use plan::{Failure, Options, PlannedMove};
use serde::Serialize;
use std::{collections::HashSet, path::PathBuf};
use tauri::{AppHandle, Manager, State};

const MODULE: &str = "organizer";

fn check_root(root: &str) -> AppResult<PathBuf> {
    let p = PathBuf::from(root);
    if !p.is_dir() {
        return Err(AppError::msg("Seçilen klasör bulunamadı. Lütfen başka bir klasör seçin."));
    }
    if p.parent().is_none() {
        return Err(AppError::msg("Disk kök dizini düzenlenemez. Lütfen içindeki bir klasörü seçin."));
    }
    Ok(p)
}

#[tauri::command]
pub fn default_downloads(app: AppHandle) -> AppResult<String> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|_| AppError::msg("İndirilenler klasörü bulunamadı. Lütfen bir klasör seçin."))?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn organizer_plan(root: String, options: Options) -> AppResult<Vec<PlannedMove>> {
    blocking(move || plan::build_plan(&check_root(&root)?, &options)).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyReport {
    pub batch_id: String,
    pub moved: usize,
    pub failed: Vec<Failure>,
}

#[tauri::command]
pub async fn organizer_apply(db: State<'_, Db>, root: String, options: Options, selected: Vec<String>) -> AppResult<ApplyReport> {
    let selected: HashSet<String> = selected.into_iter().collect();
    let outcome = blocking(move || plan::apply_moves(&check_root(&root)?, &options, &selected)).await?;
    let batch_id = history::new_batch_id();
    history::log(&db, MODULE, "move", &batch_id, &outcome.done)?;
    Ok(ApplyReport { batch_id, moved: outcome.done.len(), failed: outcome.failed })
}

#[tauri::command]
pub fn organizer_history(db: State<'_, Db>) -> AppResult<Vec<Batch>> {
    history::list(&db, MODULE)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub restored: usize,
    pub failed: Vec<Failure>,
}

#[tauri::command]
pub async fn organizer_undo(db: State<'_, Db>, batch_id: String) -> AppResult<UndoReport> {
    let rows = history::active(&db, MODULE, &batch_id)?;
    let pairs: Vec<(String, String)> = rows.iter().map(|(_, s, d)| (s.clone(), d.clone())).collect();
    let (ok, failed) = blocking(move || Ok(plan::undo_moves(&pairs))).await?;
    let ids: Vec<i64> = ok.iter().map(|i| rows[*i].0).collect();
    history::mark_undone(&db, &ids)?;
    Ok(UndoReport { restored: ok.len(), failed })
}

#[tauri::command]
pub async fn organizer_find_duplicates(root: String, recursive: bool) -> AppResult<Vec<DupGroup>> {
    blocking(move || dedupe::find_duplicates(&check_root(&root)?, recursive)).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashReport {
    pub trashed: usize,
    pub skipped: usize,
}

/// Kalıcı silme yoktur: dosyalar yalnızca Geri Dönüşüm Kutusu'na gönderilir.
#[tauri::command]
pub async fn organizer_trash_duplicates(requests: Vec<TrashRequest>) -> AppResult<TrashReport> {
    blocking(move || {
        let mut trashed = 0;
        let mut skipped = 0;
        for req in &requests {
            let verified = dedupe::verified_removals(req)?;
            skipped += req.remove.len() - verified.len();
            if verified.is_empty() {
                continue;
            }
            trash::delete_all(&verified)
                .map_err(|_| AppError::msg("Dosyalar Geri Dönüşüm Kutusu'na gönderilemedi. Dosyalar başka bir programda açık olabilir."))?;
            trashed += verified.len();
        }
        Ok(TrashReport { trashed, skipped })
    })
    .await
}
