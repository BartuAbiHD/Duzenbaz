use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Tepsi simgesinin gerçekten oluşturulup oluşturulamadığı. Bazı Linux masaüstlerinde (tepsi desteği olmayanlar)
/// oluşturulamaz; o zaman ana pencereyi kapatmak uygulamayı gizlemek yerine bitirmelidir, yoksa uygulamaya bir daha erişilemez.
pub struct TrayState(pub std::sync::atomic::AtomicBool);

impl TrayState {
    pub fn available(app: &AppHandle) -> bool {
        app.try_state::<TrayState>().map(|s| s.0.load(std::sync::atomic::Ordering::Relaxed)).unwrap_or(true)
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Düzen-Baz'ı Göster", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .tooltip("Düzen-Baz")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
