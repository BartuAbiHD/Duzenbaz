mod exif;
mod fsops;
mod rules;

use crate::{
    db::Db,
    error::{AppError, AppResult},
    history::{self, Batch},
    util::blocking,
};
use rules::{PreviewItem, Rules, Status};
use serde::Serialize;
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::State;

const MODULE: &str = "renamer";

/// Sürüklenen yolları dosya listesine çevirir: klasörün doğrudan içindeki dosyalar eklenir.
#[tauri::command]
pub async fn renamer_expand(paths: Vec<String>) -> AppResult<Vec<String>> {
    blocking(move || {
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
                if c.is_file() && !name.starts_with('.') && seen.insert(c.to_string_lossy().to_lowercase()) {
                    out.push(c.to_string_lossy().to_string());
                }
            }
        }
        Ok(out)
    })
    .await
}

#[tauri::command]
pub async fn renamer_preview(paths: Vec<String>, rules: Rules) -> AppResult<Vec<PreviewItem>> {
    blocking(move || rules::preview(&paths, &rules)).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameReport {
    pub batch_id: String,
    pub renamed: usize,
}

#[tauri::command]
pub async fn renamer_apply(db: State<'_, Db>, paths: Vec<String>, rules: Rules) -> AppResult<RenameReport> {
    let pairs = blocking(move || {
        // Ön izleme sunucuda yeniden hesaplanır; arayüzün gönderdiği yeni adlara güvenilmez.
        let items = rules::preview(&paths, &rules)?;
        if items.iter().any(|i| matches!(i.status, Status::Conflict | Status::Invalid)) {
            return Err(AppError::msg("Çakışan veya geçersiz adlar var. Önce bunları düzeltin; hiçbir dosya değiştirilmedi."));
        }
        let pairs: Vec<(PathBuf, PathBuf)> = items
            .iter()
            .filter(|i| i.status == Status::Ok)
            .map(|i| (PathBuf::from(&i.src), PathBuf::from(&i.src).with_file_name(&i.new_name)))
            .collect();
        fsops::rename_all(&pairs)?;
        Ok(pairs)
    })
    .await?;
    let batch_id = history::new_batch_id();
    let logged: Vec<(String, String)> =
        pairs.iter().map(|(s, d)| (s.to_string_lossy().to_string(), d.to_string_lossy().to_string())).collect();
    history::log(&db, MODULE, "rename", &batch_id, &logged)?;
    Ok(RenameReport { batch_id, renamed: pairs.len() })
}

#[tauri::command]
pub fn renamer_history(db: State<'_, Db>) -> AppResult<Vec<Batch>> {
    history::list(&db, MODULE)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub restored: usize,
}

#[tauri::command]
pub async fn renamer_undo(db: State<'_, Db>, batch_id: String) -> AppResult<UndoReport> {
    let rows = history::active(&db, MODULE, &batch_id)?;
    // Geri alırken yönler tersine çevrilir: yeni ad → eski ad.
    let pairs: Vec<(PathBuf, PathBuf)> = rows.iter().map(|(_, old, new)| (PathBuf::from(new), PathBuf::from(old))).collect();
    blocking(move || fsops::rename_all(&pairs)).await?;
    let ids: Vec<i64> = rows.iter().map(|r| r.0).collect();
    history::mark_undone(&db, &ids)?;
    Ok(UndoReport { restored: ids.len() })
}
