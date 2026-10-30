//! Geri alınabilir işlemlerin (taşıma, yeniden adlandırma) ortak günlüğü.
use crate::{db::Db, error::AppResult};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub batch_id: String,
    pub created_at: String,
    pub total: i64,
    pub remaining: i64,
}

pub fn new_batch_id() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S-%3f").to_string()
}

/// `pairs` = (eski yol, yeni yol)
pub fn log(db: &Db, module: &str, kind: &str, batch_id: &str, pairs: &[(String, String)]) -> AppResult<()> {
    db.with(|conn| {
        let tx = conn.transaction()?;
        for (src, dst) in pairs {
            tx.execute(
                "INSERT INTO operations (batch_id, module, kind, src, dst) VALUES (?1, ?2, ?3, ?4, ?5)",
                (batch_id, module, kind, src, dst),
            )?;
        }
        tx.commit()?;
        Ok(())
    })
}

pub fn list(db: &Db, module: &str) -> AppResult<Vec<Batch>> {
    db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT batch_id, datetime(MIN(created_at), 'localtime'), COUNT(*), SUM(CASE WHEN undone = 0 THEN 1 ELSE 0 END)
             FROM operations WHERE module = ?1 GROUP BY batch_id ORDER BY MAX(id) DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([module], |r| {
            Ok(Batch { batch_id: r.get(0)?, created_at: r.get(1)?, total: r.get(2)?, remaining: r.get(3)? })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

/// Henüz geri alınmamış satırlar: (id, eski yol, yeni yol).
pub fn active(db: &Db, module: &str, batch_id: &str) -> AppResult<Vec<(i64, String, String)>> {
    db.with(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, src, dst FROM operations WHERE module = ?1 AND batch_id = ?2 AND undone = 0 ORDER BY id")?;
        let rows = stmt.query_map([module, batch_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn mark_undone(db: &Db, ids: &[i64]) -> AppResult<()> {
    db.with(|conn| {
        let tx = conn.transaction()?;
        for id in ids {
            tx.execute("UPDATE operations SET undone = 1 WHERE id = ?1", [id])?;
        }
        tx.commit()?;
        Ok(())
    })
}
