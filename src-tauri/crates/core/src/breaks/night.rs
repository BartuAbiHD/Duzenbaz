//! Gece modu hesapları: ne zaman, ne kadar sıcak. Ekrana uygulama işi `platform.rs`'te.

/// Gece aralığının başında ve sonunda renk geçişinin sürdüğü dakika.
const FADE_MINUTES: f32 = 30.0;

pub fn parse_hhmm(s: &str) -> Option<u32> {
    let (h, m) = s.trim().split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// 0.0 = gündüz, 1.0 = tam gece. Aralık gece yarısını aşabilir (örn. 20:00 → 07:00).
/// Başlangıçta ve bitişte yumuşak geçiş uygulanır.
pub fn night_factor(now_min: f32, start: u32, end: u32) -> f32 {
    if start == end {
        return 0.0;
    }
    let (start, end) = (start as f32, end as f32);
    let duration = (end - start).rem_euclid(1440.0);
    let elapsed = (now_min - start).rem_euclid(1440.0);
    if elapsed >= duration {
        return 0.0;
    }
    (elapsed / FADE_MINUTES).min((duration - elapsed) / FADE_MINUTES).min(1.0)
}

/// `warmth` 0–100. Dönen değerler kırmızı, yeşil, mavi kanal çarpanlarıdır (0.5–1.0).
/// 0 → 6600 K (değişiklik yok), 100 → yaklaşık 3300 K. Windows, gama eğrisinde yarıdan
/// fazla kısmaya izin vermediği için çarpanlar 0.5'in altına inmez.
pub fn multipliers(warmth: f32) -> [f32; 3] {
    let t = (6600.0 - warmth.clamp(0.0, 100.0) / 100.0 * 3300.0) / 100.0;
    let red = if t <= 66.0 { 255.0 } else { 329.698_73 * (t - 60.0).powf(-0.133_204_76) };
    let green = if t <= 66.0 { 99.470_8 * t.ln() - 161.119_57 } else { 288.122_17 * (t - 60.0).powf(-0.075_514_85) };
    let blue = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };
    [red, green, blue].map(|c: f32| (c / 255.0).clamp(0.5, 1.0))
}

/// Windows'un beklediği 3×256 gama tablosu (kırmızı, yeşil, mavi sırasıyla).
pub fn ramp(mult: [f32; 3]) -> [u16; 768] {
    let mut out = [0u16; 768];
    for (c, m) in mult.iter().enumerate() {
        for i in 0..256 {
            out[c * 256 + i] = ((i as f32 * 257.0) * m).round() as u16;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_times() {
        assert_eq!(parse_hhmm("20:30"), Some(1230));
        assert_eq!(parse_hhmm("7:05"), Some(425));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("abc"), None);
    }

    #[test]
    fn factor_handles_midnight_wrap_and_fades() {
        let (start, end) = (20 * 60, 7 * 60);
        assert_eq!(night_factor(12.0 * 60.0, start, end), 0.0);
        assert_eq!(night_factor(20.0 * 60.0, start, end), 0.0);
        assert!((night_factor(20.0 * 60.0 + 15.0, start, end) - 0.5).abs() < 1e-4);
        assert_eq!(night_factor(23.0 * 60.0, start, end), 1.0);
        assert_eq!(night_factor(2.0 * 60.0, start, end), 1.0);
        assert!((night_factor(6.0 * 60.0 + 45.0, start, end) - 0.5).abs() < 1e-4);
        assert_eq!(night_factor(7.0 * 60.0, start, end), 0.0);
        // Gece yarısını aşmayan aralık
        assert_eq!(night_factor(60.0, 0, 120), 1.0 * (60.0f32 / 30.0).min(1.0));
        assert_eq!(night_factor(100.0, 5, 5), 0.0);
    }

    #[test]
    fn no_warmth_is_identity_and_more_warmth_cuts_blue_first() {
        assert_eq!(multipliers(0.0), [1.0, 1.0, 1.0]);
        let (a, b) = (multipliers(30.0), multipliers(80.0));
        assert!(a[2] > b[2], "mavi azalmalı");
        assert!(b[2] < b[1] && b[1] <= b[0], "mavi < yeşil <= kırmızı");
        for m in multipliers(100.0) {
            assert!((0.5..=1.0).contains(&m));
        }
    }

    #[test]
    fn ramp_is_monotonic_and_identity_without_warmth() {
        let id = ramp([1.0; 3]);
        assert_eq!((id[0], id[255], id[511]), (0, 65535, 65535));
        let warm = ramp(multipliers(100.0));
        for c in 0..3 {
            assert!(warm[c * 256..(c + 1) * 256].windows(2).all(|w| w[0] <= w[1]));
            // Windows en parlak değerin en az yarısını bekler.
            assert!(warm[c * 256 + 255] >= 32768);
        }
    }
}
