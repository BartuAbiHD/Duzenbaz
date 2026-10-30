//! Pencerelerin bir tarayıcı gibi davranmasını engeller: sağ tık menüsü, geliştirici araçları,
//! yenileme, yazdırma, yakınlaştırma gibi kısayollar kapatılır.
//! Geliştirme derlemelerinde Tauri'nin eklediği Ctrl+Shift+I betiği ayrıca arayüzde
//! (src/lib/lockdown.ts) engellenir. Hata ayıklamak için `DUZENBAZ_DEVTOOLS=1` ile geçici olarak açılabilir.
use tauri::{AppHandle, Manager};

pub fn apply_all(app: &AppHandle) {
    if std::env::var_os("DUZENBAZ_DEVTOOLS").is_some() {
        return;
    }
    for (_, window) in app.webview_windows() {
        apply(&window);
    }
}

#[cfg(windows)]
fn apply(window: &tauri::WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Settings3, ICoreWebView2Settings4, ICoreWebView2Settings6};
    use windows_core::Interface;

    let _ = window.with_webview(|webview| unsafe {
        let Ok(settings) = webview.controller().CoreWebView2().and_then(|core| core.Settings()) else { return };
        let _ = settings.SetAreDevToolsEnabled(false);
        let _ = settings.SetAreDefaultContextMenusEnabled(false);
        let _ = settings.SetIsZoomControlEnabled(false);
        let _ = settings.SetIsStatusBarEnabled(false);
        // F5, Ctrl+R, Ctrl+P, Ctrl+F gibi tarayıcıya özgü kısayolları kapatır;
        // Ctrl+C/V/X/A/Z gibi düzenleme kısayolları çalışmaya devam eder.
        if let Ok(s) = settings.cast::<ICoreWebView2Settings3>() {
            let _ = s.SetAreBrowserAcceleratorKeysEnabled(false);
        }
        if let Ok(s) = settings.cast::<ICoreWebView2Settings4>() {
            let _ = s.SetIsPasswordAutosaveEnabled(false);
            let _ = s.SetIsGeneralAutofillEnabled(false);
        }
        if let Ok(s) = settings.cast::<ICoreWebView2Settings6>() {
            let _ = s.SetIsSwipeNavigationEnabled(false);
        }
    });
}

#[cfg(not(windows))]
fn apply(_window: &tauri::WebviewWindow) {}
