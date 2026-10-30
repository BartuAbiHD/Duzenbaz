//! macOS platform çağrılarının (CoreGraphics, Objective-C çalışma zamanı) çökmeden çalıştığını doğrular.
//! `cargo test -p duzenbaz-core --test macos -- --ignored --nocapture`. Normal bir Mac'te yanlışlıkla çalışmasın diye
//! `#[ignore]` işaretlidir: `send_paste` öndeki uygulamaya gerçek bir Cmd+V gönderir.
#![cfg(target_os = "macos")]

use duzenbaz_core::{breaks::platform as breaks, clipboard::platform as clipboard};

// Masaüstü uygulaması AppKit'i zaten yükler; bu test ikilisinde de yüklü olsun ki NSWorkspace/NSPasteboard sınıfları bulunsun.
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[test]
#[ignore = "gerçek bir macOS oturumu gerektirir ve Cmd+V gönderir"]
fn macos_platform_calls_run_without_crashing() {
    // Boşta kalma süresi: CoreGraphics olay kaynağı sorgusu.
    let idle = breaks::idle_secs();
    println!("boşta süre: {idle} sn");
    assert!(breaks::night_supported());

    // Objective-C mesajları: yanlış seçici veya tür uyuşmazlığı burada çalışma zamanında patlar.
    println!("öndeki uygulama: {:?}", clipboard::foreground_process());
    assert!(!clipboard::excluded_by_source(), "boş panoda gizli işaret olmamalı");

    // Ekran aktarım tablosu: ekran yoksa false dönebilir, ama çökmemeli. Sonunda renkler eski haline getirilir.
    let applied = breaks::apply_warmth(40.0);
    println!("gece modu uygulandı: {applied}");
    breaks::reset_colors();

    // CGEvent gönderimi: izin yoksa olaylar yok sayılır, ama çağrı çökmemeli.
    clipboard::send_paste();
}
