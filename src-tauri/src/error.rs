use serde::{Serialize, Serializer};
use std::io::ErrorKind;

/// Arayüze giden tüm hatalar, kullanıcının anlayacağı Türkçe bir mesaja çevrilir.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
}

impl AppError {
    pub fn msg(text: impl Into<String>) -> Self {
        AppError::Msg(text.into())
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::Db(_) => "Yerel veritabanına erişilemedi. Uygulamayı yeniden başlatmayı deneyin.".into(),
            AppError::Io(e) => match e.kind() {
                ErrorKind::NotFound => "Dosya veya klasör bulunamadı. Taşınmış ya da silinmiş olabilir.".into(),
                ErrorKind::PermissionDenied => {
                    "Bu dosya veya klasöre erişim izniniz yok. Başka bir programda açık olabilir.".into()
                }
                ErrorKind::AlreadyExists => "Aynı adda bir dosya zaten var.".into(),
                _ => "Dosya işlemi tamamlanamadı. Lütfen tekrar deneyin.".into(),
            },
            AppError::Msg(m) => m.clone(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.user_message())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_errors_are_turkish() {
        let e = AppError::from(std::io::Error::from(ErrorKind::PermissionDenied));
        assert!(e.user_message().contains("izniniz yok"));
    }
}
