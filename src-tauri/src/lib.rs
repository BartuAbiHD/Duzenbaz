mod breaks;
mod clipboard;
mod lockdown;
mod organizer;
mod renamer;
mod tray;
mod util;

// Çekirdek paketteki ortak modüller; uygulamanın geri kalanı bunlara `crate::db` gibi yollarla erişir.
pub use duzenbaz_core::{db, error, history};

use error::AppResult;
use std::sync::atomic::AtomicBool;
use tauri::{Manager, RunEvent, State, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::ShortcutState;

#[tauri::command]
fn db_version(db: State<db::Db>) -> AppResult<i64> {
    db.version()
}

/// Uygulama kendini kurabilir mi? Windows ve macOS'ta evet; Linux'ta .deb kendini güncelleyemez, yalnızca bildirim verilir.
#[tauri::command]
fn app_can_self_update() -> bool {
    cfg!(any(windows, target_os = "macos"))
}

#[tauri::command]
fn app_restart(app: tauri::AppHandle) {
    app.restart();
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        clipboard::popup::toggle(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // CLI ile aynı kural: DUZENBAZ_DATA_DIR verilmişse veri orada tutulur (ayrı bir kopyayı denemek için).
            let dir = match std::env::var_os("DUZENBAZ_DATA_DIR") {
                Some(d) => std::path::PathBuf::from(d),
                None => app.path().app_data_dir()?,
            };
            std::fs::create_dir_all(&dir)?;
            app.manage(db::Db::open(&dir.join("duzenbaz.db"))?);

            // Anahtar alınamazsa uygulama yine açılır; yalnızca pano geçmişi devre dışı kalır.
            app.manage(clipboard::Clip { vault: Default::default(), shortcut_ok: AtomicBool::new(false) });
            clipboard::load_vault(app.handle().clone());
            clipboard::popup::create(app.handle())?;
            let settings = app.state::<db::Db>().with(|c| clipboard::store::load_settings(c)).unwrap_or_default();
            let registered = clipboard::popup::register_shortcut(app.handle(), &settings.shortcut).is_ok();
            app.state::<clipboard::Clip>().shortcut_ok.store(registered, std::sync::atomic::Ordering::Relaxed);
            clipboard::watcher::start(app.handle().clone());

            let breaks = breaks::initial(&app.state::<db::Db>());
            app.manage(breaks);
            breaks::create_window(app.handle())?;
            breaks::start(app.handle().clone());

            // Tepsi oluşturulamazsa uygulama yine açılır (bkz. tray::TrayState).
            let tray_ok = match tray::create(app.handle()) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("Sistem tepsisi oluşturulamadı: {e}");
                    false
                }
            };
            app.manage(tray::TrayState(AtomicBool::new(tray_ok)));
            lockdown::apply_all(app.handle());

            // Oturum açılırken pencere açılmadan doğrudan tepsiye gider (tepsi yoksa pencere görünür kalır).
            if tray_ok && std::env::args().any(|a| a == "--minimized") {
                if let Some(w) = app.get_webview_window("main") {
                    w.hide()?;
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Pencereyi kapatmak uygulamayı bitirmez, tepsiye indirir. Tepsi yoksa ana pencereyi kapatmak uygulamayı bitirir.
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" && !tray::TrayState::available(window.app_handle()) {
                    window.app_handle().exit(0);
                } else {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
            // Açılır pano penceresi odağı kaybedince kendiliğinden kapanır.
            WindowEvent::Focused(false) if window.label() == clipboard::popup::LABEL => {
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            db_version,
            app_can_self_update,
            app_restart,
            organizer::default_downloads,
            organizer::organizer_plan,
            organizer::organizer_apply,
            organizer::organizer_history,
            organizer::organizer_undo,
            organizer::organizer_find_duplicates,
            organizer::organizer_trash_duplicates,
            renamer::renamer_expand,
            renamer::renamer_preview,
            renamer::renamer_apply,
            renamer::renamer_history,
            renamer::renamer_undo,
            clipboard::clip_list,
            clipboard::clip_image,
            clipboard::clip_copy,
            clipboard::clip_pin,
            clipboard::clip_set_meta,
            clipboard::clip_delete,
            clipboard::clip_clear,
            clipboard::clip_hide_popup,
            clipboard::clip_get_settings,
            clipboard::clip_set_settings,
            breaks::breaks_get_settings,
            breaks::breaks_set_settings,
            breaks::breaks_get_state,
            breaks::breaks_start_now,
            breaks::breaks_skip,
            breaks::breaks_snooze,
            breaks::breaks_preview_night,
            breaks::breaks_report
        ])
        .build(tauri::generate_context!())
        .expect("Düzen-Baz başlatılamadı")
        .run(|app, event| match event {
            RunEvent::Exit => breaks::on_exit(app),
            // macOS: pencere tepsiye indirilmişken Dock simgesine tıklanınca ana pencere yeniden açılır.
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => tray::show_main(app),
            _ => {}
        });
}
