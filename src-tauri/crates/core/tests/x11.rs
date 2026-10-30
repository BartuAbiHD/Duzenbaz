//! Linux/X11 platform çağrılarının gerçek bir X sunucusuyla konuştuğunu doğrular.
//! Sanal bir sunucuda çalıştırılır: `xvfb-run -a cargo test -p duzenbaz-core --test x11 -- --ignored --nocapture`.
//! `send_paste` sahte klavye girdisi ürettiği için normal masaüstünde yanlışlıkla çalışmasın diye `#[ignore]` işaretlidir.
#![cfg(target_os = "linux")]

use duzenbaz_core::{breaks::platform as breaks, clipboard::platform as clipboard};
use std::{thread::sleep, time::Duration};
use x11rb::{
    connection::Connection,
    protocol::randr::ConnectionExt as _,
};

#[test]
#[ignore = "bir X sunucusu gerektirir (xvfb-run ile çalıştırın)"]
fn x11_idle_night_mode_and_key_injection_talk_to_the_server() {
    assert!(std::env::var_os("DISPLAY").is_some(), "DISPLAY yok; xvfb-run ile çalıştırın");
    std::env::remove_var("XDG_SESSION_TYPE");
    assert!(breaks::night_supported(), "X11 oturumunda gece modu desteklenmeli");

    // Hiç girdi olmadığı için boşta kalma süresi artmalı: XScreenSaver sorgusu gerçekten çalışıyor mu?
    sleep(Duration::from_millis(2500));
    let idle = breaks::idle_secs();
    assert!(idle >= 2, "XScreenSaver boşta süresi {idle} sn döndü; en az 2 bekleniyordu");

    // XTest ile Ctrl+V göndermek sunucuya girdi olarak ulaşmalı ve boşta süresini sıfırlamalı.
    clipboard::send_paste();
    sleep(Duration::from_millis(400));
    let after = breaks::idle_secs();
    assert!(after <= 1, "sahte Ctrl+V sonrası boşta süresi {after} sn; XTest çalışmıyor olabilir");

    // Pencere yöneticisi olmayan sanal sunucuda etkin pencere yoktur; çökmeden `None` dönmeli.
    println!("öndeki program: {:?}", clipboard::foreground_process());
    assert!(!clipboard::excluded_by_source(), "Linux'ta pano işareti okunmaz");

    // Gece modu: sunucuda gama destekli bir ekran denetleyicisi varsa etkisini doğrudan sunucudan oku.
    let applied = breaks::apply_warmth(60.0);
    let (conn, screen) = x11rb::connect(None).expect("X sunucusuna bağlanılamadı");
    let root = conn.setup().roots[screen].root;
    let resources = conn.randr_get_screen_resources_current(root).unwrap().reply().unwrap();
    let mut checked = false;
    for crtc in resources.crtcs {
        let size = conn.randr_get_crtc_gamma_size(crtc).unwrap().reply().unwrap().size;
        if size < 2 {
            continue;
        }
        let gamma = conn.randr_get_crtc_gamma(crtc).unwrap().reply().unwrap();
        let last = size as usize - 1;
        assert!(applied, "gama destekli ekran var ama apply_warmth false döndü");
        assert!(gamma.blue[last] < gamma.red[last], "mavi kanal kırmızıdan düşük olmalı: {} / {}", gamma.blue[last], gamma.red[last]);
        checked = true;
    }
    println!("gece modu uygulandı: {applied}, etkisi doğrulandı: {checked}");
    breaks::reset_colors();
}
