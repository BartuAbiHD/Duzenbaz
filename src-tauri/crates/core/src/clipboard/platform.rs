//! Panoya özgü işletim sistemi çağrıları.
//!
//! - Windows: pano biçimleri, `GetForegroundWindow`, `SendInput`.
//! - macOS: NSPasteboard işaretleri, NSWorkspace, CoreGraphics olayları (Erişilebilirlik izni gerekir).
//! - Linux: yalnızca X11 oturumları (etkin pencere ve XTest). Wayland, uygulamaların başka pencerenin
//!   kimliğini öğrenmesine ve tuş göndermesine izin vermez; orada bu üç işlev "yok" gibi davranır.

pub use imp::{excluded_by_source, foreground_process, send_paste};

/// Hata ayıklama: `DUZENBAZ_DEBUG=1` ile yapıştırma adımları standart hataya yazılır.
pub fn debug(msg: &str) {
    if std::env::var_os("DUZENBAZ_DEBUG").is_some() {
        eprintln!("pano: {msg}");
    }
}

/// Pano penceresi açılmadan önce etkin pencereyi hatırlar; seçimden sonra `restore_foreground_window` ona odağı geri verir.
/// Yalnızca Linux/X11'de gereklidir: pencere yöneticisi, gizlenen pencerenin ardından odağı her zaman önceki pencereye
/// vermez ve Ctrl+V boşa gider. Windows ve macOS bunu kendiliğinden yapar.
#[cfg(target_os = "linux")]
pub use imp::{remember_foreground_window, restore_foreground_window};
#[cfg(not(target_os = "linux"))]
pub fn remember_foreground_window() {}
#[cfg(not(target_os = "linux"))]
pub fn restore_foreground_window() {}

#[cfg(windows)]
mod imp {
    use clipboard_win::{raw, register_format, Clipboard};
    use std::mem::size_of;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION},
        UI::{
            Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    };

    fn format_id(name: &str) -> Option<u32> {
        register_format(name).map(|f| f.get())
    }

    /// Parola yöneticileri ve Windows'un kendi kuralı: kaynak uygulama içeriğin geçmişe
    /// alınmamasını istiyorsa pano verisine işaret koyar.
    pub fn excluded_by_source() -> bool {
        if let Some(f) = format_id("ExcludeClipboardContentFromMonitorProcessing") {
            if raw::is_format_avail(f) {
                return true;
            }
        }
        for name in ["CanIncludeInClipboardHistory", "CanUploadToCloudClipboard"] {
            let Some(f) = format_id(name) else { continue };
            if !raw::is_format_avail(f) {
                continue;
            }
            if let Ok(_open) = Clipboard::new_attempts(5) {
                let mut buf = Vec::new();
                if raw::get_vec(f, &mut buf).is_ok() && buf.len() >= 4 && buf[..4] == [0, 0, 0, 0] {
                    return true;
                }
            }
        }
        false
    }

    /// Önde olan programın exe adı (küçük harf), örn. "keepass.exe".
    pub fn foreground_process() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return None;
            }
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return None;
            }
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            if ok == 0 {
                return None;
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            path.rsplit(['\\', '/']).next().map(|s| s.to_lowercase())
        }
    }

    fn key(vk: u16, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: if up { KEYEVENTF_KEYUP } else { 0 }, time: 0, dwExtraInfo: 0 },
            },
        }
    }

    /// Önde olan pencereye Ctrl+V gönderir.
    pub fn send_paste() {
        const V: u16 = 0x56;
        let inputs = [key(VK_CONTROL, false), key(V, false), key(V, true), key(VK_CONTROL, true)];
        unsafe {
            SendInput(inputs.len() as u32, inputs.as_ptr(), size_of::<INPUT>() as i32);
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use objc2::{
        msg_send,
        runtime::{AnyClass, AnyObject},
    };
    use std::ffi::{c_char, c_void, CStr};
    use std::ptr::null;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(source: *const c_void, virtual_key: u16, key_down: bool) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    /// `kVK_ANSI_V`
    const KEY_V: u16 = 0x09;
    /// `kCGEventFlagMaskCommand`
    const FLAG_COMMAND: u64 = 0x0010_0000;
    /// `kCGHIDEventTap`
    const HID_EVENT_TAP: u32 = 0;

    /// NSString → Rust metni. `s` boş (nil) olabilir.
    unsafe fn text(s: *mut AnyObject) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let utf8: *const c_char = msg_send![s, UTF8String];
        if utf8.is_null() {
            None
        } else {
            Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
        }
    }

    /// Parola yöneticilerinin kullandığı nspasteboard.org işaretleri: "gizli" ve "geçici" içerik.
    pub fn excluded_by_source() -> bool {
        unsafe {
            let Some(class) = AnyClass::get(c"NSPasteboard") else { return false };
            let pasteboard: *mut AnyObject = msg_send![class, generalPasteboard];
            if pasteboard.is_null() {
                return false;
            }
            let types: *mut AnyObject = msg_send![pasteboard, types];
            if types.is_null() {
                return false;
            }
            let count: usize = msg_send![types, count];
            (0..count).any(|i| {
                let t: *mut AnyObject = msg_send![types, objectAtIndex: i];
                matches!(
                    text(t).as_deref(),
                    Some("org.nspasteboard.ConcealedType") | Some("org.nspasteboard.TransientType")
                )
            })
        }
    }

    /// Önde olan uygulamanın çalıştırılabilir dosya adı (küçük harf), örn. "1password".
    pub fn foreground_process() -> Option<String> {
        unsafe {
            let class = AnyClass::get(c"NSWorkspace")?;
            let workspace: *mut AnyObject = msg_send![class, sharedWorkspace];
            if workspace.is_null() {
                return None;
            }
            let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return None;
            }
            let url: *mut AnyObject = msg_send![app, executableURL];
            if url.is_null() {
                return None;
            }
            let name: *mut AnyObject = msg_send![url, lastPathComponent];
            text(name).map(|s| s.to_lowercase())
        }
    }

    /// Cmd+V gönderir. macOS, uygulamaya "Erişilebilirlik" izni verilmediyse bu olayları sessizce yok sayar.
    pub fn send_paste() {
        unsafe {
            for key_down in [true, false] {
                let event = CGEventCreateKeyboardEvent(null(), KEY_V, key_down);
                if event.is_null() {
                    continue;
                }
                CGEventSetFlags(event, FLAG_COMMAND);
                CGEventPost(HID_EVENT_TAP, event);
                CFRelease(event as *const c_void);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use x11rb::{
        connection::Connection,
        protocol::{
            xproto::{AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, KEY_PRESS_EVENT, KEY_RELEASE_EVENT},
            xtest::ConnectionExt as _,
        },
        rust_connection::RustConnection,
    };

    fn is_wayland() -> bool {
        std::env::var("XDG_SESSION_TYPE").map(|v| v.eq_ignore_ascii_case("wayland")).unwrap_or(false)
    }

    fn connect() -> Option<(RustConnection, usize)> {
        if is_wayland() || std::env::var_os("DISPLAY").is_none() {
            return None;
        }
        x11rb::connect(None).ok()
    }

    /// Linux'ta parola yöneticisi işareti okunmaz (panodaki `x-kde-passwordManagerHint` hedefi);
    /// bunun yerine Ayarlar'daki "yoksayılan programlar" listesi kullanılır.
    pub fn excluded_by_source() -> bool {
        false
    }

    fn atom(conn: &RustConnection, name: &[u8]) -> Option<u32> {
        conn.intern_atom(false, name).ok()?.reply().ok().map(|r| r.atom)
    }

    /// Etkin pencere (`_NET_ACTIVE_WINDOW`).
    fn active_window(conn: &RustConnection, root: u32) -> Option<u32> {
        let active = atom(conn, b"_NET_ACTIVE_WINDOW")?;
        let reply = conn.get_property(false, root, active, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
        let window = reply.value32()?.next();
        window.filter(|w| *w != 0)
    }

    static PREVIOUS_WINDOW: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    pub fn remember_foreground_window() {
        let Some((conn, screen)) = connect() else { return };
        let window = active_window(&conn, conn.setup().roots[screen].root).unwrap_or(0);
        super::debug(&format!("önceki etkin pencere: {window:#x}"));
        PREVIOUS_WINDOW.store(window, std::sync::atomic::Ordering::Relaxed);
    }

    /// Pencere yöneticisine "şu pencereyi etkinleştir" der (`_NET_ACTIVE_WINDOW` istemci iletisi, kaynak: pager).
    pub fn restore_foreground_window() {
        let window = PREVIOUS_WINDOW.load(std::sync::atomic::Ordering::Relaxed);
        if window == 0 {
            return super::debug("odak geri verilemedi: önceki pencere bilinmiyor");
        }
        let Some((conn, screen)) = connect() else { return super::debug("odak geri verilemedi: X sunucusuna bağlanılamadı") };
        let root = conn.setup().roots[screen].root;
        let Some(active) = atom(&conn, b"_NET_ACTIVE_WINDOW") else { return };
        super::debug(&format!("odak geri veriliyor: {window:#x}"));
        let event = ClientMessageEvent::new(32, window, active, [2, 0, 0, 0, 0]);
        let _ = conn.send_event(false, root, EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, event);
        let _ = conn.flush();
    }

    /// Etkin pencerenin işlem kimliği (`_NET_ACTIVE_WINDOW` → `_NET_WM_PID`).
    fn active_window_pid(conn: &RustConnection, root: u32) -> Option<u32> {
        let window = active_window(conn, root)?;
        let pid_atom = atom(conn, b"_NET_WM_PID")?;
        let reply = conn.get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1).ok()?.reply().ok()?;
        // Değer önce bir değişkene alınır: yineleyici `reply`'dan ödünç alır ve fonksiyon sonunda ondan önce ölmelidir.
        let pid = reply.value32()?.next();
        pid
    }

    /// Önde olan programın dosya adı (küçük harf), örn. "keepassxc". Wayland'de `None`.
    pub fn foreground_process() -> Option<String> {
        let (conn, screen) = connect()?;
        let pid = active_window_pid(&conn, conn.setup().roots[screen].root)?;
        let from_exe = std::fs::read_link(format!("/proc/{pid}/exe"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
        // Bazı paketlenmiş uygulamalarda (snap/flatpak) `exe` okunamaz; `comm` her zaman vardır (15 karaktere kısalır).
        from_exe.or_else(|| std::fs::read_to_string(format!("/proc/{pid}/comm")).ok().map(|s| s.trim().to_lowercase()))
    }

    /// Ctrl+V gönderir (XTest). Wayland'de yapılamaz, hiçbir şey yapmaz.
    pub fn send_paste() {
        let Some((conn, screen)) = connect() else { return super::debug("Ctrl+V gönderilemedi: X sunucusuna bağlanılamadı (Wayland oturumu olabilir)") };
        let setup = conn.setup();
        let root = setup.roots[screen].root;
        let (min, max) = (setup.min_keycode, setup.max_keycode);
        let Some(map) = conn.get_keyboard_mapping(min, max - min + 1).ok().and_then(|c| c.reply().ok()) else { return };
        let per_key = map.keysyms_per_keycode.max(1) as usize;
        let keycode_of = |keysym: u32| -> Option<u8> {
            map.keysyms.chunks(per_key).position(|syms| syms.contains(&keysym)).map(|i| min + i as u8)
        };
        // XK_Control_L = 0xffe3, XK_v = 0x0076
        let (Some(ctrl), Some(v)) = (keycode_of(0xffe3), keycode_of(0x0076)) else { return super::debug("Ctrl+V gönderilemedi: klavye düzeninde Ctrl veya V tuşu bulunamadı") };
        for (keycode, event) in [(ctrl, KEY_PRESS_EVENT), (v, KEY_PRESS_EVENT), (v, KEY_RELEASE_EVENT), (ctrl, KEY_RELEASE_EVENT)] {
            if let Err(e) = conn.xtest_fake_input(event, keycode, 0, root, 0, 0, 0) {
                super::debug(&format!("tuş gönderilemedi: {e}"));
            }
        }
        let _ = conn.flush();
        super::debug("Ctrl+V gönderildi");
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod imp {
    pub fn excluded_by_source() -> bool {
        false
    }
    pub fn foreground_process() -> Option<String> {
        None
    }
    pub fn send_paste() {}
}
