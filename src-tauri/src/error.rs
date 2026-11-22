// SPDX-License-Identifier: GPL-3.0-or-later
//! Hata tipleri. Her hata kararlı bir kod taşır; arayüz kodu Türkçe karşılığını
//! ve çözüm butonunu bu koda göre seçer.

use serde::ser::{Serialize, SerializeStruct, Serializer};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Dosya veya klasör bulunamadı: {0}")]
    NotFound(String),
    #[error("Bu konuma erişim izni yok")]
    PermissionDenied,
    #[error("Konum salt okunur")]
    ReadOnly,
    #[error("Aynı adda bir öğe zaten var: {0}")]
    AlreadyExists(String),
    #[error("Geçersiz ad: {0}")]
    InvalidName(String),
    #[error("Depolama alanı dolu")]
    StorageFull,
    #[error("Dosya okunamadı: {0}")]
    Unreadable(String),
    #[error("İşlem iptal edildi")]
    Cancelled,
    #[error("Arşiv hatası: {0}")]
    Archive(String),
    #[error("Bu dosya türü desteklenmiyor: {0}")]
    Unsupported(String),
    #[error("Bu özellik yalnızca Android'de çalışır")]
    AndroidOnly,
    #[error("İşlem başarısız: {0}")]
    Io(String),
    #[error("Beklenmeyen hata: {0}")]
    Internal(String),
}

impl AppError {
    /// Arayüzün eşleştirdiği kararlı hata kodu.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ReadOnly => "READ_ONLY",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::InvalidName(_) => "INVALID_NAME",
            Self::StorageFull => "STORAGE_FULL",
            Self::Unreadable(_) => "UNREADABLE",
            Self::Cancelled => "CANCELLED",
            Self::Archive(_) => "ARCHIVE_ERROR",
            Self::Unsupported(_) => "UNSUPPORTED",
            Self::AndroidOnly => "ANDROID_ONLY",
            Self::Io(_) => "IO_ERROR",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists(err.to_string()),
            std::io::ErrorKind::StorageFull => Self::StorageFull,
            std::io::ErrorKind::InvalidInput => Self::InvalidName(err.to_string()),
            _ => Self::Io(err.to_string()),
        }
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::FileNotFound => Self::NotFound("arşiv".into()),
            other => Self::Archive(other.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(format!("JSON: {err}"))
    }
}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        Self::Unreadable(format!("görsel: {err}"))
    }
}

/// Frontend'e dönen gövde: `{ code, message }`.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_hatalari_koda_eslesir() {
        let err = AppError::from(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "yok",
        ));
        assert_eq!(err.code(), "PERMISSION_DENIED");
        assert_eq!(
            AppError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "x")).code(),
            "NOT_FOUND"
        );
    }

    #[test]
    fn serde_cikisi_kod_ve_mesaj_icerir() {
        let json = serde_json::to_value(AppError::InvalidName("..".into())).unwrap();
        assert_eq!(json["code"], "INVALID_NAME");
        assert!(json["message"].as_str().unwrap().contains(".."));
    }
}
