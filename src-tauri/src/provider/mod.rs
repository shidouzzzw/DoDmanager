// SPDX-License-Identifier: GPL-3.0-or-later
//! `StorageProvider` katmanı: yerel dosya sistemi ve SAF aynı arayüzü paylaşır.
//! Oluşturma/kopyalama işlemleri "hedef klasör + ad" alır; böylece SAF
//! URI'lerinin iç yapısı hiçbir zaman arayüzde veya komutlarda taklit edilmez.

pub mod local;
pub mod saf;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::models::{FileEntry, StorageInfo};
use crate::util;

pub use local::LocalProvider;
pub use saf::SafProvider;

/// İptal bayrağı; iş parçacıkları arasında paylaşılır.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn guard(&self) -> AppResult<()> {
        if self.is_cancelled() {
            Err(AppError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// İlerleme bildirimi; arayüz event'lerini bu kapanış üretir.
#[derive(Clone)]
pub struct ProgressSink {
    reporter: Arc<dyn Fn(u64) + Send + Sync>,
}

impl ProgressSink {
    pub fn new(reporter: impl Fn(u64) + Send + Sync + 'static) -> Self {
        Self {
            reporter: Arc::new(reporter),
        }
    }

    pub fn add(&self, bytes: u64) {
        (self.reporter)(bytes);
    }

    pub fn silent() -> Self {
        Self::new(|_| {})
    }
}

/// İki erişim modelinin ortak yüzeyi.
pub trait StorageProvider: Send + Sync {
    fn list(&self, dir: &str) -> AppResult<Vec<FileEntry>>;
    fn stat(&self, path: &str) -> AppResult<FileEntry>;
    fn is_dir(&self, path: &str) -> AppResult<bool>;
    fn read(&self, path: &str, max_bytes: u64) -> AppResult<Vec<u8>>;
    fn write(&self, path: &str, data: &[u8]) -> AppResult<()>;
    /// Yeni klasör; oluşturulan öğenin yolunu döndürür.
    fn create_dir(&self, parent: &str, name: &str) -> AppResult<String>;
    /// Yeni boş dosya; oluşturulan öğenin yolunu döndürür.
    fn create_file(&self, parent: &str, name: &str) -> AppResult<String>;
    fn remove(&self, path: &str, is_dir: bool) -> AppResult<()>;
    /// Yeniden adlandırma; yeni yolu döndürür.
    fn rename(&self, path: &str, new_name: &str) -> AppResult<String>;
    fn copy(
        &self,
        source: &str,
        dest_dir: &str,
        dest_name: &str,
        cancel: &CancelToken,
        progress: &ProgressSink,
        transferred: &mut u64,
    ) -> AppResult<String>;
    fn move_entry(
        &self,
        source: &str,
        dest_dir: &str,
        dest_name: &str,
        cancel: &CancelToken,
        progress: &ProgressSink,
        transferred: &mut u64,
    ) -> AppResult<String>;

    fn storage_info(&self, _path: &str) -> AppResult<StorageInfo> {
        Err(AppError::Unsupported("depolama bilgisi".into()))
    }
}

/// Kullanıcıya gösterilecek sağlayıcı adı.
pub fn provider_name(path: &str) -> &'static str {
    if util::is_content_uri(path) {
        "saf"
    } else {
        "local"
    }
}

pub fn open<'a>(app: &'a AppHandle, path: &str) -> AppResult<Box<dyn StorageProvider + 'a>> {
    if util::is_content_uri(path) {
        Ok(Box::new(SafProvider::new(app)?))
    } else {
        Ok(Box::new(LocalProvider))
    }
}

/// Yol tabanlı yardımcılar; aynı sağlayıcıyı iki kez açmamak için.
pub fn list(app: &AppHandle, dir: &str) -> AppResult<Vec<FileEntry>> {
    open(app, dir)?.list(dir)
}

pub fn stat(app: &AppHandle, path: &str) -> AppResult<FileEntry> {
    open(app, path)?.stat(path)
}

pub fn is_dir(app: &AppHandle, path: &str) -> AppResult<bool> {
    open(app, path)?.is_dir(path)
}

pub fn remove(app: &AppHandle, path: &str) -> AppResult<()> {
    let is_dir = is_dir(app, path)?;
    open(app, path)?.remove(path, is_dir)
}

pub fn create_dir(app: &AppHandle, parent: &str, name: &str) -> AppResult<String> {
    util::validate_name(name)?;
    open(app, parent)?.create_dir(parent, name)
}

pub fn create_file(app: &AppHandle, parent: &str, name: &str) -> AppResult<String> {
    util::validate_name(name)?;
    open(app, parent)?.create_file(parent, name)
}

pub fn rename(app: &AppHandle, path: &str, new_name: &str) -> AppResult<String> {
    util::validate_name(new_name)?;
    open(app, path)?.rename(path, new_name)
}

/// Bir klasördeki var olan adlar; benzersiz ad üretimi için.
pub fn existing_names(app: &AppHandle, dir: &str) -> AppResult<Vec<String>> {
    Ok(list(app, dir)?.into_iter().map(|e| e.name).collect())
}

/// Çakışmasız hedef adı üretir: `foto.jpg` -> `foto (1).jpg`.
pub fn unique_name_in(app: &AppHandle, dir: &str, name: &str) -> AppResult<String> {
    let names = existing_names(app, dir)?;
    Ok(util::unique_name(&names, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saglayici_adi_semadan_secelir() {
        assert_eq!(provider_name("/storage/emulated/0"), "local");
        assert_eq!(
            provider_name("content://com.android.externalstorage.documents/tree/primary%3A"),
            "saf"
        );
    }

    #[test]
    fn iptal_bayragi_paylasilir() {
        let token = CancelToken::new();
        let clone = token.clone();
        assert!(token.guard().is_ok());
        clone.cancel();
        assert!(token.is_cancelled());
        assert_eq!(token.guard().unwrap_err().code(), "CANCELLED");
    }

    #[test]
    fn ilerleme_sink_biriktirir() {
        use std::sync::atomic::{AtomicU64, Ordering};
        let total = Arc::new(AtomicU64::new(0));
        let counter = total.clone();
        let sink = ProgressSink::new(move |n| {
            counter.fetch_add(n, Ordering::SeqCst);
        });
        sink.add(10);
        sink.add(32);
        assert_eq!(total.load(Ordering::SeqCst), 42);
    }
}
