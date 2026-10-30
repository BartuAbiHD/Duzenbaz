//! Toplu Adlandırıcı'nın Tauri komutları. Asıl iş `duzenbaz_core::renamer::service` içinde yapılır.
use crate::{db::Db, error::AppResult, history::Batch, util::blocking};
use duzenbaz_core::renamer::{
    rules::{PreviewItem, Rules},
    service::{self, RenameReport, UndoReport},
};
use tauri::State;

/// Sürüklenen yolları dosya listesine çevirir: klasörün doğrudan içindeki dosyalar eklenir.
#[tauri::command]
pub async fn renamer_expand(paths: Vec<String>) -> AppResult<Vec<String>> {
    blocking(move || service::expand(&paths)).await
}

#[tauri::command]
pub async fn renamer_preview(paths: Vec<String>, rules: Rules) -> AppResult<Vec<PreviewItem>> {
    blocking(move || service::preview(&paths, &rules)).await
}

#[tauri::command]
pub async fn renamer_apply(db: State<'_, Db>, paths: Vec<String>, rules: Rules) -> AppResult<RenameReport> {
    let db = db.inner().clone();
    blocking(move || service::apply(&db, &paths, &rules)).await
}

#[tauri::command]
pub fn renamer_history(db: State<'_, Db>) -> AppResult<Vec<Batch>> {
    service::list_history(&db)
}

#[tauri::command]
pub async fn renamer_undo(db: State<'_, Db>, batch_id: String) -> AppResult<UndoReport> {
    let db = db.inner().clone();
    blocking(move || service::undo(&db, &batch_id)).await
}
