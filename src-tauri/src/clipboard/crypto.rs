use crate::error::{AppError, AppResult};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};

const SERVICE: &str = "com.duzenbaz.app";
const ACCOUNT: &str = "clipboard-key";
const NONCE_LEN: usize = 12;

/// Pano içeriğini veritabanına yazmadan önce şifreler. Anahtar işletim sisteminin
/// güvenli deposunda (Windows Kimlik Bilgisi Yöneticisi) tutulur, veritabanı dosyasında bulunmaz.
pub struct Vault {
    cipher: ChaCha20Poly1305,
    mac_key: [u8; 32],
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn key_error() -> AppError {
    AppError::msg("Pano geçmişi için şifreleme anahtarına erişilemedi. İşletim sisteminin parola deposu kullanılamıyor olabilir.")
}

impl Vault {
    pub fn from_key(key: [u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(Key::from_slice(&key)),
            // Aynı anahtarı hem şifreleme hem özet için kullanmamak için ayrı bir anahtar türetilir.
            mac_key: blake3::derive_key("duzenbaz clipboard dedupe hash", &key),
        }
    }

    pub fn load() -> AppResult<Self> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT).map_err(|_| key_error())?;
        match entry.get_password() {
            Ok(h) => Ok(Self::from_key(unhex(&h).ok_or_else(key_error)?)),
            Err(keyring::Error::NoEntry) => {
                let key: [u8; 32] = ChaCha20Poly1305::generate_key(&mut OsRng).into();
                entry.set_password(&hex(&key)).map_err(|_| key_error())?;
                Ok(Self::from_key(key))
            }
            Err(_) => Err(key_error()),
        }
    }

    pub fn seal(&self, plain: &[u8]) -> AppResult<Vec<u8>> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = self.cipher.encrypt(&nonce, plain).map_err(|_| AppError::msg("Veri şifrelenemedi."))?;
        let mut out = nonce.to_vec();
        out.extend(ct);
        Ok(out)
    }

    /// Anahtar yanlışsa veya veri bozulmuşsa `None` döner.
    pub fn open(&self, blob: &[u8]) -> Option<Vec<u8>> {
        if blob.len() < NONCE_LEN + 16 {
            return None;
        }
        self.cipher.decrypt(Nonce::from_slice(&blob[..NONCE_LEN]), &blob[NONCE_LEN..]).ok()
    }

    /// Aynı içeriği tanımak için anahtarlı özet; içerikten geri çözülemez.
    pub fn mac(&self, data: &[u8]) -> String {
        blake3::keyed_hash(&self.mac_key, data).to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_tamper_detection() {
        let v = Vault::from_key([1; 32]);
        let blob = v.seal(b"merhaba").unwrap();
        assert!(!blob.windows(7).any(|w| w == b"merhaba"));
        assert_eq!(v.open(&blob).unwrap(), b"merhaba");

        let mut bad = blob.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert!(v.open(&bad).is_none());
        assert!(Vault::from_key([2; 32]).open(&blob).is_none(), "başka anahtarla açılmamalı");
    }

    #[test]
    fn nonces_differ_and_mac_is_stable() {
        let v = Vault::from_key([1; 32]);
        assert_ne!(v.seal(b"x").unwrap(), v.seal(b"x").unwrap());
        assert_eq!(v.mac(b"x"), v.mac(b"x"));
        assert_ne!(v.mac(b"x"), Vault::from_key([2; 32]).mac(b"x"));
    }

    #[test]
    fn hex_roundtrip() {
        let k = [0xabu8; 32];
        assert_eq!(unhex(&hex(&k)), Some(k));
        assert_eq!(unhex("zz"), None);
    }
}
