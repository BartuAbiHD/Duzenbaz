//! Panoya özgü işletim sistemi çağrıları. Windows dışında sade yedekler kullanılır.

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

#[cfg(not(windows))]
mod imp {
    pub fn excluded_by_source() -> bool {
        false
    }
    pub fn foreground_process() -> Option<String> {
        None
    }
    pub fn send_paste() {}
}

pub use imp::*;
