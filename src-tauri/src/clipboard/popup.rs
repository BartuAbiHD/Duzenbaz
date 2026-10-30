use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub const LABEL: &str = "popup";

/// Kısayolla açılan küçük pano penceresi. Başlangıçta gizli oluşturulur.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=popup".into()))
        .title("Pano Geçmişi")
        .inner_size(440.0, 560.0)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .center()
        .build()?;
    Ok(())
}

pub fn toggle(app: &AppHandle) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
    } else {
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit_to(LABEL, "popup-shown", ());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
}

pub fn register_shortcut(app: &AppHandle, accelerator: &str) -> AppResult<()> {
    let shortcut: Shortcut =
        accelerator.parse().map_err(|_| AppError::msg("Kısayol biçimi geçersiz. Örnek: Ctrl+Shift+V"))?;
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.register(shortcut)
        .map_err(|_| AppError::msg("Bu kısayol başka bir program tarafından kullanılıyor. Lütfen farklı bir kısayol seçin."))
}
