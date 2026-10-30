//! Dosya Düzenleyici'nin Tauri komutları. Asıl iş `duzenbaz_core::organizer::service` içinde yapılır.
use crate::{
    db::Db,
    error::{AppError, AppResult},
    history::Batch,
    util::blocking,
};
use duzenbaz_core::organizer::{
    dedupe::{DupGroup, TrashRequest},
    plan::{Options, PlannedMove},
    service::{self, ApplyReport, TrashReport, UndoReport},
};
use std::collections::HashSet;
use tauri::{AppHandle, Manager, State};

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
    blocking(move || service::preview(&root, &options)).await
}

#[tauri::command]
pub async fn organizer_apply(db: State<'_, Db>, root: String, options: Options, selected: Vec<String>) -> AppResult<ApplyReport> {
    let db = db.inner().clone();
    let selected: HashSet<String> = selected.into_iter().collect();
    blocking(move || service::apply(&db, &root, &options, &selected)).await
}

#[tauri::command]
pub fn organizer_history(db: State<'_, Db>) -> AppResult<Vec<Batch>> {
    service::list_history(&db)
}

#[tauri::command]
pub async fn organizer_undo(db: State<'_, Db>, batch_id: String) -> AppResult<UndoReport> {
    let db = db.inner().clone();
    blocking(move || service::undo(&db, &batch_id)).await
}

#[tauri::command]
pub async fn organizer_find_duplicates(root: String, recursive: bool) -> AppResult<Vec<DupGroup>> {
    blocking(move || service::find_duplicates(&root, recursive)).await
}

/// Kalıcı silme yoktur: dosyalar yalnızca Geri Dönüşüm Kutusu'na gönderilir.
#[tauri::command]
pub async fn organizer_trash_duplicates(requests: Vec<TrashRequest>) -> AppResult<TrashReport> {
    blocking(move || service::trash_duplicates(&requests)).await
}
