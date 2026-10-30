//! Boşta kalma süresi ve ekran renk ayarı (gece modu).
//!
//! - Windows: `GetLastInputInfo` ve gama tablosu.
//! - macOS: CoreGraphics (son olaydan bu yana geçen süre, ekran aktarım tablosu).
//! - Linux: yalnızca X11 oturumları (XScreenSaver ve RandR gama). Wayland'de boşta kalma hep sıfır
//!   döner (kullanıcı hep aktif sayılır) ve gece modu desteklenmez; çünkü Wayland ekran rengine
//!   ve girdi süresine uygulamaların erişmesine izin vermez.

pub use imp::{idle_secs, night_supported};

/// Ekranı `warmth` (0–100) oranında sıcak tona çevirir; 0 eski haline getirir. En az bir ekran başarılıysa `true`.
pub fn apply_warmth(warmth: f32) -> bool {
    imp::set_warmth(warmth)
}

pub fn reset_colors() {
    imp::set_warmth(0.0);
}

#[cfg(windows)]
mod imp {
    use crate::breaks::night;
    use std::{ffi::c_void, mem::size_of, ptr::null};
    use windows_sys::Win32::{
        Graphics::Gdi::{CreateDCW, DeleteDC, EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP},
        System::SystemInformation::GetTickCount,
        UI::{
            ColorSystem::SetDeviceGammaRamp,
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
        },
    };

    pub fn night_supported() -> bool {
        true
    }

    /// Son klavye/fare girdisinden bu yana geçen saniye.
    pub fn idle_secs() -> u32 {
        let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        unsafe {
            if GetLastInputInfo(&mut info) == 0 {
                return 0;
            }
            GetTickCount().wrapping_sub(info.dwTime) / 1000
        }
    }

    /// Masaüstüne bağlı tüm ekranlara gama tablosunu uygular. En az bir ekran başarılıysa true.
    fn set_ramp(ramp: &[u16; 768]) -> bool {
        let display: Vec<u16> = "DISPLAY\0".encode_utf16().collect();
        let mut any = false;
        let mut index = 0;
        loop {
            let mut dev: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
            dev.cb = size_of::<DISPLAY_DEVICEW>() as u32;
            if unsafe { EnumDisplayDevicesW(null(), index, &mut dev, 0) } == 0 {
                break;
            }
            index += 1;
            if dev.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
                continue;
            }
            unsafe {
                let dc = CreateDCW(display.as_ptr(), dev.DeviceName.as_ptr(), null(), null());
                if dc.is_null() {
                    continue;
                }
                if SetDeviceGammaRamp(dc, ramp.as_ptr() as *const c_void) != 0 {
                    any = true;
                }
                DeleteDC(dc);
            }
        }
        any
    }

    pub fn set_warmth(warmth: f32) -> bool {
        set_ramp(&night::ramp(night::multipliers(warmth)))
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use crate::breaks::night;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
        fn CGGetActiveDisplayList(max_displays: u32, active_displays: *mut u32, display_count: *mut u32) -> i32;
        fn CGSetDisplayTransferByTable(display: u32, table_size: u32, red: *const f32, green: *const f32, blue: *const f32) -> i32;
        fn CGDisplayRestoreColorSyncSettings();
    }

    /// `kCGEventSourceStateCombinedSessionState`
    const COMBINED_SESSION_STATE: i32 = 0;
    /// `kCGAnyInputEventType`: `(CGEventType)~0`
    const ANY_INPUT_EVENT: u32 = u32::MAX;
    const TABLE_SIZE: usize = 256;
    const MAX_DISPLAYS: usize = 16;

    pub fn night_supported() -> bool {
        true
    }

    pub fn idle_secs() -> u32 {
        let secs = unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
        if secs.is_finite() && secs > 0.0 {
            secs as u32
        } else {
            0
        }
    }

    pub fn set_warmth(warmth: f32) -> bool {
        if warmth <= 0.0 {
            unsafe { CGDisplayRestoreColorSyncSettings() };
            return true;
        }
        let mult = night::multipliers(warmth);
        let table = |m: f32| -> Vec<f32> { (0..TABLE_SIZE).map(|i| i as f32 / (TABLE_SIZE - 1) as f32 * m).collect() };
        let (red, green, blue) = (table(mult[0]), table(mult[1]), table(mult[2]));

        let mut displays = [0u32; MAX_DISPLAYS];
        let mut count = 0u32;
        if unsafe { CGGetActiveDisplayList(MAX_DISPLAYS as u32, displays.as_mut_ptr(), &mut count) } != 0 {
            return false;
        }
        let mut any = false;
        for id in &displays[..(count as usize).min(MAX_DISPLAYS)] {
            let status = unsafe { CGSetDisplayTransferByTable(*id, TABLE_SIZE as u32, red.as_ptr(), green.as_ptr(), blue.as_ptr()) };
            any |= status == 0;
        }
        any
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use crate::breaks::night;
    use x11rb::{
        connection::Connection,
        protocol::{randr::ConnectionExt as _, screensaver::ConnectionExt as _},
        rust_connection::RustConnection,
    };

    fn is_wayland() -> bool {
        std::env::var("XDG_SESSION_TYPE").map(|v| v.eq_ignore_ascii_case("wayland")).unwrap_or(false)
    }

    /// X11 oturumunda (veya yalnızca X11 desteği olan bir masaüstünde) çalışır.
    pub fn night_supported() -> bool {
        !is_wayland() && std::env::var_os("DISPLAY").is_some()
    }

    fn connect() -> Option<(RustConnection, usize)> {
        if !night_supported() {
            return None;
        }
        x11rb::connect(None).ok()
    }

    pub fn idle_secs() -> u32 {
        let Some((conn, screen)) = connect() else { return 0 };
        let root = conn.setup().roots[screen].root;
        conn.screensaver_query_info(root)
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .map(|reply| reply.ms_since_user_input / 1000)
            .unwrap_or(0)
    }

    /// Her etkin ekran denetleyicisinin (CRTC) gama tablosunu ayarlar.
    pub fn set_warmth(warmth: f32) -> bool {
        let Some((conn, screen)) = connect() else { return false };
        let root = conn.setup().roots[screen].root;
        let Some(resources) = conn.randr_get_screen_resources_current(root).ok().and_then(|c| c.reply().ok()) else {
            return false;
        };
        let mult = night::multipliers(warmth);
        let mut any = false;
        for crtc in resources.crtcs {
            let size = conn.randr_get_crtc_gamma_size(crtc).ok().and_then(|c| c.reply().ok()).map(|r| r.size as usize).unwrap_or(0);
            if size < 2 {
                continue; // kapalı ekran
            }
            let ramp = |m: f32| -> Vec<u16> { (0..size).map(|i| (i as f32 / (size - 1) as f32 * 65535.0 * m).round() as u16).collect() };
            let ok = conn
                .randr_set_crtc_gamma(crtc, &ramp(mult[0]), &ramp(mult[1]), &ramp(mult[2]))
                .ok()
                .map(|cookie| cookie.check().is_ok())
                .unwrap_or(false);
            any |= ok;
        }
        let _ = conn.flush();
        any
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod imp {
    pub fn night_supported() -> bool {
        false
    }
    pub fn idle_secs() -> u32 {
        0
    }
    pub fn set_warmth(_warmth: f32) -> bool {
        false
    }
}
