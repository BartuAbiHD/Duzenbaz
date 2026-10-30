use super::crypto::Vault;
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Parola yöneticileri. Adlar uzantısız yazılır; karşılaştırma `.exe` uzantısını ve büyük/küçük harfi yok sayar,
/// böylece aynı liste Windows (keepass.exe), macOS (KeePassXC) ve Linux (keepassxc) için geçerlidir.
pub const DEFAULT_IGNORED_APPS: &[&str] = &[
    "keepass",
    "keepassxc",
    "keepass2",
    "keeweb",
    "bitwarden",
    "1password",
    "lastpass",
    "dashlane",
    "enpass",
    "nordpass",
    "roboform",
    "strongbox",
];

fn normalized_app_name(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower.strip_suffix(".exe").map(str::to_string).unwrap_or(lower)
}

/// `process` (örn. "KeePass.exe", "keepassxc") yoksayma listesindeki bir girdiyle eşleşiyor mu.
pub fn is_ignored_app(ignored: &[String], process: &str) -> bool {
    let process = normalized_app_name(process);
    ignored.iter().any(|a| normalized_app_name(a) == process)
}

/// Linux'ta Ctrl+Shift+V terminallerde "yapıştır"dır; genel kısayol onu çalmasın diye farklı bir varsayılan kullanılır.
#[cfg(target_os = "linux")]
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+V";
#[cfg(not(target_os = "linux"))]
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+V";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClipSettings {
    pub paused: bool,
    pub max_items: usize,
    pub ignored_apps: Vec<String>,
    pub shortcut: String,
    pub auto_paste: bool,
}

impl Default for ClipSettings {
    fn default() -> Self {
        Self {
            paused: false,
            max_items: 500,
            ignored_apps: DEFAULT_IGNORED_APPS.iter().map(|s| s.to_string()).collect(),
            shortcut: DEFAULT_SHORTCUT.into(),
            auto_paste: true,
        }
    }
}

pub fn load_settings(conn: &Connection) -> AppResult<ClipSettings> {
    let v: Option<String> =
        conn.query_row("SELECT value FROM settings WHERE key = 'clip.settings'", [], |r| r.get(0)).optional()?;
    Ok(v.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default())
}

pub fn save_settings(conn: &Connection, s: &ClipSettings) -> AppResult<()> {
    let json = serde_json::to_string(s).map_err(|_| AppError::msg("Ayarlar kaydedilemedi."))?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('clip.settings', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [json],
    )?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Meta {
    category: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: i64,
    pub kind: String,
    pub preview: String,
    pub category: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_ms: i64,
    pub size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

pub struct NewClip<'a> {
    pub kind: &'a str,
    pub bytes: &'a [u8],
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub fn detect_kind(text: &str) -> &'static str {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();
    let looks_like_url = (lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www."))
        && !t.contains(char::is_whitespace);
    if looks_like_url {
        "link"
    } else {
        "text"
    }
}

fn open_meta(v: &Vault, blob: &[u8]) -> Meta {
    v.open(blob).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn seal_meta(v: &Vault, m: &Meta) -> AppResult<Vec<u8>> {
    v.seal(&serde_json::to_vec(m).map_err(|_| AppError::msg("Etiketler kaydedilemedi."))?)
}

/// Yeni içerik ekler. Aynı içerik zaten varsa yeniden eklemez, listenin başına taşır.
/// Dönen bool: gerçekten yeni bir kayıt mı oluşturuldu.
pub fn upsert(conn: &mut Connection, v: &Vault, c: &NewClip, now_ms: i64, max_items: usize) -> AppResult<(i64, bool)> {
    let hash = v.mac(c.bytes);
    let existing: Option<i64> = conn.query_row("SELECT id FROM clip_items WHERE hash = ?1", [&hash], |r| r.get(0)).optional()?;
    if let Some(id) = existing {
        conn.execute("UPDATE clip_items SET updated_ms = ?1 WHERE id = ?2", params![now_ms, id])?;
        return Ok((id, false));
    }
    conn.execute(
        "INSERT INTO clip_items (kind, content, meta, hash, size, width, height, updated_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![c.kind, v.seal(c.bytes)?, seal_meta(v, &Meta::default())?, hash, c.bytes.len() as i64, c.width, c.height, now_ms],
    )?;
    let id = conn.last_insert_rowid();
    prune(conn, max_items)?;
    Ok((id, true))
}

/// Sabitlenmemiş en eski kayıtları siler; sabitlenenlere dokunmaz.
pub fn prune(conn: &Connection, max_items: usize) -> AppResult<()> {
    conn.execute(
        "DELETE FROM clip_items WHERE pinned = 0 AND id NOT IN
         (SELECT id FROM clip_items WHERE pinned = 0 ORDER BY updated_ms DESC LIMIT ?1)",
        [max_items as i64],
    )?;
    Ok(())
}

pub struct Listing {
    pub items: Vec<ClipItem>,
    pub categories: Vec<String>,
}

/// İçerik şifreli olduğu için arama bellekte, çözülmüş metin üzerinde yapılır.
pub fn list(conn: &Connection, v: &Vault, query: &str, category: &str, limit: usize) -> AppResult<Listing> {
    let q = query.trim().to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT id, kind, CASE WHEN kind = 'image' THEN NULL ELSE content END, meta, pinned, size, width, height, updated_ms
         FROM clip_items ORDER BY pinned DESC, updated_ms DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<Vec<u8>>>(2)?,
            r.get::<_, Vec<u8>>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, Option<i64>>(6)?,
            r.get::<_, Option<i64>>(7)?,
            r.get::<_, i64>(8)?,
        ))
    })?;

    let mut items = Vec::new();
    let mut categories = BTreeSet::new();
    for row in rows {
        let (id, kind, content, meta_blob, pinned, size, width, height, updated_ms) = row?;
        let meta = open_meta(v, &meta_blob);
        if !meta.category.is_empty() {
            categories.insert(meta.category.clone());
        }
        let text = content.and_then(|c| v.open(&c)).map(|b| String::from_utf8_lossy(&b).to_string());
        let matches_query = q.is_empty()
            || text.as_deref().is_some_and(|t| t.to_lowercase().contains(&q))
            || meta.category.to_lowercase().contains(&q)
            || meta.tags.iter().any(|t| t.to_lowercase().contains(&q));
        let matches_category = category.is_empty() || meta.category == category;
        if !(matches_query && matches_category) || items.len() >= limit {
            continue;
        }
        let preview = match (&kind[..], &text) {
            ("image", _) => String::new(),
            (_, Some(t)) => t.chars().take(500).collect(),
            (_, None) => "(içerik çözülemedi)".into(),
        };
        items.push(ClipItem { id, kind, preview, category: meta.category, tags: meta.tags, pinned: pinned != 0, updated_ms, size, width, height });
    }
    Ok(Listing { items, categories: categories.into_iter().collect() })
}

pub fn get_content(conn: &Connection, v: &Vault, id: i64) -> AppResult<(String, Vec<u8>)> {
    let row: Option<(String, Vec<u8>)> =
        conn.query_row("SELECT kind, content FROM clip_items WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
    let (kind, blob) = row.ok_or_else(|| AppError::msg("Pano öğesi bulunamadı. Silinmiş olabilir."))?;
    let bytes = v.open(&blob).ok_or_else(|| AppError::msg("Pano öğesinin içeriği çözülemedi."))?;
    Ok((kind, bytes))
}

pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> AppResult<()> {
    conn.execute("UPDATE clip_items SET pinned = ?1 WHERE id = ?2", params![pinned as i64, id])?;
    Ok(())
}

pub fn set_meta(conn: &Connection, v: &Vault, id: i64, category: &str, tags: &[String]) -> AppResult<()> {
    let mut seen = BTreeSet::new();
    let tags: Vec<String> =
        tags.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty() && seen.insert(t.to_lowercase())).take(10).collect();
    let meta = Meta { category: category.trim().chars().take(40).collect(), tags };
    conn.execute("UPDATE clip_items SET meta = ?1 WHERE id = ?2", params![seal_meta(v, &meta)?, id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM clip_items WHERE id = ?1", [id])?;
    Ok(())
}

pub fn clear(conn: &Connection, keep_pinned: bool) -> AppResult<usize> {
    let n = if keep_pinned {
        conn.execute("DELETE FROM clip_items WHERE pinned = 0", [])?
    } else {
        conn.execute("DELETE FROM clip_items", [])?
    };
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Connection, Vault) {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrate(&mut conn).unwrap();
        (conn, Vault::from_key([9; 32]))
    }

    fn add(conn: &mut Connection, v: &Vault, text: &str, now: i64, max: usize) -> (i64, bool) {
        upsert(conn, v, &NewClip { kind: detect_kind(text), bytes: text.as_bytes(), width: None, height: None }, now, max).unwrap()
    }

    #[test]
    fn content_is_encrypted_at_rest() {
        let (mut conn, v) = setup();
        add(&mut conn, &v, "gizli-parola-123", 1, 10);
        let raw: Vec<u8> = conn.query_row("SELECT content FROM clip_items", [], |r| r.get(0)).unwrap();
        assert!(!raw.windows(16).any(|w| w == b"gizli-parola-123"));
        let hash: String = conn.query_row("SELECT hash FROM clip_items", [], |r| r.get(0)).unwrap();
        assert!(!hash.contains("gizli"));
    }

    #[test]
    fn duplicates_bump_instead_of_inserting() {
        let (mut conn, v) = setup();
        let (a, new_a) = add(&mut conn, &v, "bir", 1, 10);
        add(&mut conn, &v, "iki", 2, 10);
        let (a2, new_a2) = add(&mut conn, &v, "bir", 3, 10);
        assert!(new_a && !new_a2 && a == a2);
        let l = list(&conn, &v, "", "", 10).unwrap();
        assert_eq!(l.items.len(), 2);
        assert_eq!(l.items[0].preview, "bir");
    }

    #[test]
    fn pinned_first_and_survive_pruning() {
        let (mut conn, v) = setup();
        let (old, _) = add(&mut conn, &v, "eski", 1, 2);
        set_pinned(&conn, old, true).unwrap();
        for (i, t) in ["a", "b", "c", "d"].iter().enumerate() {
            add(&mut conn, &v, t, 10 + i as i64, 2);
        }
        let l = list(&conn, &v, "", "", 10).unwrap();
        let previews: Vec<_> = l.items.iter().map(|i| i.preview.as_str()).collect();
        assert_eq!(previews, vec!["eski", "d", "c"]);
    }

    #[test]
    fn search_covers_text_category_and_tags_and_kind_detection() {
        let (mut conn, v) = setup();
        let (id, _) = add(&mut conn, &v, "https://ornek.com/sayfa", 1, 10);
        add(&mut conn, &v, "Alışveriş listesi: süt", 2, 10);
        set_meta(&conn, &v, id, "Çalışma", &["Dokümantasyon".into(), "dokümantasyon".into(), " ".into()]).unwrap();

        assert_eq!(list(&conn, &v, "SÜT", "", 10).unwrap().items.len(), 1, "arama büyük/küçük harfe duyarsız olmalı");
        assert_eq!(list(&conn, &v, "süt", "", 10).unwrap().items.len(), 1);
        assert_eq!(list(&conn, &v, "dokümantasyon", "", 10).unwrap().items[0].id, id);
        let by_cat = list(&conn, &v, "", "Çalışma", 10).unwrap();
        assert_eq!(by_cat.items.len(), 1);
        assert_eq!(by_cat.items[0].tags, vec!["Dokümantasyon"]);
        assert_eq!(by_cat.items[0].kind, "link");
        assert_eq!(by_cat.categories, vec!["Çalışma"]);
        assert_eq!(detect_kind("http://a.com bosluk var"), "text");
    }

    #[test]
    fn ignored_apps_match_across_platforms() {
        let list: Vec<String> = DEFAULT_IGNORED_APPS.iter().map(|s| s.to_string()).collect();
        assert!(is_ignored_app(&list, "KeePass.exe"), "Windows: uzantılı ad");
        assert!(is_ignored_app(&list, "KeePassXC"), "macOS: büyük harfli çalıştırılabilir ad");
        assert!(is_ignored_app(&list, "keepassxc"), "Linux");
        assert!(!is_ignored_app(&list, "notepad.exe"));
        // Eski sürümlerde kaydedilmiş, uzantılı girdiler de çalışmaya devam eder.
        assert!(is_ignored_app(&["Bitwarden.EXE".to_string()], "bitwarden"));
        assert!(!is_ignored_app(&[], "keepass"));
    }

    #[test]
    fn delete_clear_and_settings_roundtrip() {
        let (mut conn, v) = setup();
        let (a, _) = add(&mut conn, &v, "a", 1, 10);
        let (b, _) = add(&mut conn, &v, "b", 2, 10);
        set_pinned(&conn, b, true).unwrap();
        assert_eq!(clear(&conn, true).unwrap(), 1);
        assert!(get_content(&conn, &v, a).is_err());
        assert_eq!(get_content(&conn, &v, b).unwrap().1, b"b");
        delete(&conn, b).unwrap();
        assert_eq!(list(&conn, &v, "", "", 10).unwrap().items.len(), 0);

        let mut s = ClipSettings::default();
        s.max_items = 77;
        save_settings(&conn, &s).unwrap();
        assert_eq!(load_settings(&conn).unwrap().max_items, 77);
    }
}
