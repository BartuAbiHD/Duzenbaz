use super::{platform, store, Clip};
use crate::db::Db;
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::{borrow::Cow, io, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager};

const MAX_TEXT_BYTES: usize = 1_000_000;
const MAX_IMAGE_BYTES: usize = 40_000_000;

struct Handler(AppHandle);

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        capture(&self.0);
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, _error: io::Error) -> CallbackResult {
        CallbackResult::Next
    }
}

/// Arka planda panoyu dinler. Dinleyici herhangi bir nedenle durursa kısa bir beklemeden sonra yeniden başlar.
pub fn start(app: AppHandle) {
    thread::spawn(move || loop {
        if let Ok(mut master) = Master::new(Handler(app.clone())) {
            let _ = master.run();
        }
        thread::sleep(Duration::from_secs(2));
    });
}

pub fn encode_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, width as u32, height as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        enc.write_header().ok()?.write_image_data(rgba).ok()?;
    }
    Some(out)
}

enum Captured {
    Text(String),
    Image { width: usize, height: usize, rgba: Vec<u8> },
}

fn read_clipboard() -> Option<Captured> {
    // Değişim bildirimi geldiği anda pano başka bir program tarafından kilitli olabilir; birkaç kez denenir.
    for attempt in 0..4 {
        if attempt > 0 {
            thread::sleep(Duration::from_millis(40));
        }
        let Ok(mut cb) = arboard::Clipboard::new() else { continue };
        if let Ok(text) = cb.get_text() {
            return Some(Captured::Text(text));
        }
        if let Ok(img) = cb.get_image() {
            return Some(Captured::Image { width: img.width, height: img.height, rgba: img.bytes.into_owned() });
        }
    }
    None
}

fn capture(app: &AppHandle) {
    let db = app.state::<Db>();
    let clip = app.state::<Clip>();
    let Some(vault) = clip.vault.as_ref() else { return };
    let Ok(settings) = db.with(|c| store::load_settings(c)) else { return };
    if settings.paused || platform::excluded_by_source() {
        return;
    }
    if let Some(proc) = platform::foreground_process() {
        if settings.ignored_apps.iter().any(|a| a.eq_ignore_ascii_case(&proc)) {
            return;
        }
    }

    let (kind, bytes, dims): (&str, Cow<[u8]>, Option<(u32, u32)>) = match read_clipboard() {
        Some(Captured::Text(t)) => {
            if t.trim().is_empty() || t.len() > MAX_TEXT_BYTES {
                return;
            }
            (store::detect_kind(&t), Cow::Owned(t.into_bytes()), None)
        }
        Some(Captured::Image { width, height, rgba }) => {
            if rgba.len() > MAX_IMAGE_BYTES {
                return;
            }
            let Some(png) = encode_png(width, height, &rgba) else { return };
            ("image", Cow::Owned(png), Some((width as u32, height as u32)))
        }
        None => return,
    };

    let now_ms = chrono::Utc::now().timestamp_millis();
    let new = store::NewClip { kind, bytes: &bytes, width: dims.map(|d| d.0), height: dims.map(|d| d.1) };
    if db.with(|c| store::upsert(c, vault, &new, now_ms, settings.max_items)).is_ok() {
        let _ = app.emit("clipboard-changed", ());
    }
}
