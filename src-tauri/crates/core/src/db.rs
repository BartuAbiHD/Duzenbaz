use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

/// Sürüm i + 1 için betik `MIGRATIONS[i]`. Yeni modüller buraya yeni tablo ekler.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
     CREATE TABLE operations (
        id INTEGER PRIMARY KEY,
        batch_id TEXT NOT NULL,
        module TEXT NOT NULL,
        kind TEXT NOT NULL,
        src TEXT NOT NULL,
        dst TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        undone INTEGER NOT NULL DEFAULT 0
     );
     CREATE INDEX idx_operations_batch ON operations(batch_id);",
    // content ve meta şifreli saklanır (bkz. clipboard/crypto.rs); hash anahtarlı özettir.
    "CREATE TABLE clip_items (
        id INTEGER PRIMARY KEY,
        kind TEXT NOT NULL,
        content BLOB NOT NULL,
        meta BLOB NOT NULL,
        hash TEXT NOT NULL UNIQUE,
        pinned INTEGER NOT NULL DEFAULT 0,
        size INTEGER NOT NULL,
        width INTEGER,
        height INTEGER,
        updated_ms INTEGER NOT NULL
     );
     CREATE INDEX idx_clip_order ON clip_items(pinned DESC, updated_ms DESC);",
    // Ekran süresi saat saat tutulur (yalnızca süre; hangi programın kullanıldığı kaydedilmez).
    "CREATE TABLE screen_time (
        day TEXT NOT NULL,
        hour INTEGER NOT NULL,
        seconds INTEGER NOT NULL,
        PRIMARY KEY (day, hour)
     );
     CREATE TABLE break_log (
        day TEXT PRIMARY KEY,
        taken INTEGER NOT NULL DEFAULT 0,
        skipped INTEGER NOT NULL DEFAULT 0
     );",
];

/// Uygulamanın (Tauri) ve komut satırı aracının ortak kullandığı veri klasörü.
/// Tauri'nin `app_data_dir()` değeriyle aynıdır; `DUZENBAZ_DATA_DIR` ile değiştirilebilir.
pub fn default_data_dir() -> AppResult<PathBuf> {
    if let Some(dir) = std::env::var_os("DUZENBAZ_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_dir()
        .map(|d| d.join("com.duzenbaz.app"))
        .ok_or_else(|| AppError::msg("Veri klasörü bulunamadı."))
}

/// Veritabanı bağlantısı. Çoğaltılabilir (`Arc`), böylece arka plan iş parçacıklarına taşınabilir.
#[derive(Clone)]
pub struct Db(pub Arc<Mutex<Connection>>);

impl Db {
    pub fn open(path: &Path) -> AppResult<Self> {
        let mut conn = Connection::open(path)?;
        // Uygulama ve komut satırı aynı dosyayı aynı anda kullanabilir: kısa süre bekle, okuyanlar yazanı engellemesin.
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get::<_, String>(0))?;
        migrate(&mut conn)?;
        Ok(Db(Arc::new(Mutex::new(conn))))
    }

    /// Ortak veri klasöründeki veritabanını açar (yoksa oluşturur).
    pub fn open_default() -> AppResult<Self> {
        let dir = default_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        Self::open(&dir.join("duzenbaz.db"))
    }

    /// Yalnızca testler için bellekte geçici veritabanı.
    pub fn in_memory() -> AppResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        Ok(Db(Arc::new(Mutex::new(conn))))
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut Connection) -> AppResult<T>) -> AppResult<T> {
        let mut conn = self.0.lock().map_err(|_| AppError::msg("Veritabanı kilitlenmiş durumda."))?;
        f(&mut conn)
    }

    pub fn version(&self) -> AppResult<i64> {
        self.with(|conn| Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?))
    }
}

pub fn migrate(conn: &mut Connection) -> AppResult<()> {
    let current: usize = conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? as usize;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (i + 1) as i64)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_and_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
        let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, MIGRATIONS.len() as i64);
    }
}
