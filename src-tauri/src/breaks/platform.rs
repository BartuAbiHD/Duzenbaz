//! Boşta kalma süresi ve ekran renk ayarı. Gece modu şimdilik yalnızca Windows'ta çalışır.

#[cfg(windows)]
mod imp {
    use std::{ffi::c_void, mem::size_of, ptr::null};
    use windows_sys::Win32::{
        Graphics::Gdi::{CreateDCW, DeleteDC, EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP},
        System::SystemInformation::GetTickCount,
        UI::{
            ColorSystem::SetDeviceGammaRamp,
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
        },
    };

    pub const NIGHT_SUPPORTED: bool = true;

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
    pub fn set_ramp(ramp: &[u16; 768]) -> bool {
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
}

#[cfg(not(windows))]
mod imp {
    pub const NIGHT_SUPPORTED: bool = false;
    pub fn idle_secs() -> u32 {
        0
    }
    pub fn set_ramp(_ramp: &[u16; 768]) -> bool {
        false
    }
}

pub use imp::*;

pub fn apply_warmth(warmth: f32) -> bool {
    set_ramp(&super::night::ramp(super::night::multipliers(warmth)))
}

pub fn reset_colors() {
    apply_warmth(0.0);
}
