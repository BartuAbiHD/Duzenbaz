pub mod crypto;
pub mod platform;
pub mod popup;
pub mod store;
pub mod watcher;

use crate::{
    db::Db,
    error::{AppError, AppResult},
};
use base64::Engine;
use crypto::Vault;
use serde::Serialize;
use std::{
    borrow::Cow,
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use store::{ClipItem, ClipSettings};
use tauri::{AppHandle, State};

pub struct Clip {
    pub vault: Option<Vault>,
    pub shortcut_ok: AtomicBool,
}

impl Clip {
    fn vault(&self) -> AppResult<&Vault> {
        self.vault.as_ref().ok_or_else(|| {
            AppError::msg("Pano geçmişi kullanılamıyor: şifreleme anahtarına erişilemedi. Uygulamayı yeniden başlatmayı deneyin.")
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    items: Vec<ClipItem>,
    categories: Vec<String>,
}

#[tauri::command]
pub fn clip_list(db: State<Db>, clip: State<Clip>, query: String, category: String) -> AppResult<ListResponse> {
    let v = clip.vault()?;
    let l = db.with(|c| store::list(c, v, &query, &category, 300))?;
    Ok(ListResponse { items: l.items, categories: l.categories })
}

#[tauri::command]
pub fn clip_image(db: State<Db>, clip: State<Clip>, id: i64) -> AppResult<String> {
    let v = clip.vault()?;
    let (_, bytes) = db.with(|c| store::get_content(c, v, id))?;
    Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn decode_png(bytes: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let mut reader = png::Decoder::new(Cursor::new(bytes)).read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((info.width as usize, info.height as usize, buf))
}

/// Öğeyi panoya koyar. `paste` doğruysa pencereyi gizleyip önceki programa Ctrl+V gönderir.
#[tauri::command]
pub fn clip_copy(app: AppHandle, db: State<Db>, clip: State<Clip>, id: i64, paste: bool) -> AppResult<()> {
    let v = clip.vault()?;
    let (kind, bytes) = db.with(|c| store::get_content(c, v, id))?;
    let mut cb = arboard::Clipboard::new().map_err(|_| AppError::msg("Pano şu anda kullanılamıyor. Lütfen tekrar deneyin."))?;
    let result = if kind == "image" {
        let (width, height, rgba) = decode_png(&bytes).ok_or_else(|| AppError::msg("Görsel okunamadı."))?;
        cb.set_image(arboard::ImageData { width, height, bytes: Cow::Owned(rgba) })
    } else {
        cb.set_text(String::from_utf8_lossy(&bytes).to_string())
    };
    result.map_err(|_| AppError::msg("İçerik panoya kopyalanamadı. Pano başka bir program tarafından kullanılıyor olabilir."))?;

    if paste {
        popup::hide(&app);
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(150));
            platform::send_paste();
        });
    }
    Ok(())
}

#[tauri::command]
pub fn clip_pin(db: State<Db>, id: i64, pinned: bool) -> AppResult<()> {
    db.with(|c| store::set_pinned(c, id, pinned))
}

#[tauri::command]
pub fn clip_set_meta(db: State<Db>, clip: State<Clip>, id: i64, category: String, tags: Vec<String>) -> AppResult<()> {
    let v = clip.vault()?;
    db.with(|c| store::set_meta(c, v, id, &category, &tags))
}

#[tauri::command]
pub fn clip_delete(db: State<Db>, id: i64) -> AppResult<()> {
    db.with(|c| store::delete(c, id))
}

#[tauri::command]
pub fn clip_clear(db: State<Db>, keep_pinned: bool) -> AppResult<usize> {
    db.with(|c| store::clear(c, keep_pinned))
}

#[tauri::command]
pub fn clip_hide_popup(app: AppHandle) {
    popup::hide(&app);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
    settings: ClipSettings,
    shortcut_active: bool,
}

#[tauri::command]
pub fn clip_get_settings(db: State<Db>, clip: State<Clip>) -> AppResult<SettingsResponse> {
    Ok(SettingsResponse { settings: db.with(|c| store::load_settings(c))?, shortcut_active: clip.shortcut_ok.load(Ordering::Relaxed) })
}

#[tauri::command]
pub fn clip_set_settings(app: AppHandle, db: State<Db>, clip: State<Clip>, settings: ClipSettings) -> AppResult<SettingsResponse> {
    let mut s = settings;
    s.max_items = s.max_items.clamp(50, 5000);
    s.ignored_apps = s.ignored_apps.iter().map(|a| a.trim().to_lowercase()).filter(|a| !a.is_empty()).collect();
    s.shortcut = s.shortcut.trim().to_string();

    let old = db.with(|c| store::load_settings(c))?;
    if s.shortcut != old.shortcut || !clip.shortcut_ok.load(Ordering::Relaxed) {
        if let Err(e) = popup::register_shortcut(&app, &s.shortcut) {
            // Yeni kısayol alınamadı: eskisini geri yüklemeyi dene ve ayarı kaydetme.
            clip.shortcut_ok.store(popup::register_shortcut(&app, &old.shortcut).is_ok(), Ordering::Relaxed);
            return Err(e);
        }
        clip.shortcut_ok.store(true, Ordering::Relaxed);
    }
    db.with(|c| {
        store::save_settings(c, &s)?;
        store::prune(c, s.max_items)
    })?;
    Ok(SettingsResponse { settings: s, shortcut_active: clip.shortcut_ok.load(Ordering::Relaxed) })
}
