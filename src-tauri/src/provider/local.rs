// SPDX-License-Identifier: GPL-3.0-or-later
//! Yerel dosya sistemi sağlayıcısı. Yalnızca `std::fs` kullanır; Tauri'ye bağımlı
//! değildir, bu yüzden birim testleri masaüstünde de çalışır.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::models::{FileEntry, StorageInfo};
use crate::provider::{CancelToken, ProgressSink, StorageProvider};
use crate::util;

pub const CHUNK: usize = 256 * 1024;
/// Aynı dosya sisteminde olmadığında `rename` bu hatayı verir (EXDEV).
const EXDEV: i32 = 18;

/// Mutlak yol tabanlı sağlayıcı (tüm dosyalara erişim modu ve uygulama klasörü).
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalProvider;

impl LocalProvider {
    pub fn entry_from_path(path: &Path) -> AppResult<FileEntry> {
        let meta = fs::metadata(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let mut entry = FileEntry::build(
            path.to_string_lossy().to_string(),
            name,
            meta.is_dir(),
            if meta.is_dir() { 0 } else { meta.len() },
            meta.modified()
                .map(util::system_time_to_secs)
                .unwrap_or_default(),
        );
        entry.read_only = meta.permissions().readonly();
        Ok(entry)
    }

    fn join(parent: &str, name: &str) -> AppResult<PathBuf> {
        util::validate_name(name)?;
        let dir = util::validate_local_path(parent)?;
        Ok(dir.join(name))
    }
}

impl StorageProvider for LocalProvider {
    fn list(&self, dir: &str) -> AppResult<Vec<FileEntry>> {
        let path = util::validate_local_path(dir)?;
        let mut entries = Vec::new();
        for item in fs::read_dir(&path)? {
            // Tek bir öğe okunamazsa listeyi düşürme.
            let Ok(item) = item else { continue };
            if let Ok(entry) = LocalProvider::entry_from_path(&item.path()) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    fn stat(&self, path: &str) -> AppResult<FileEntry> {
        let p = util::validate_local_path(path)?;
        if !p.exists() {
            return Err(AppError::NotFound(path.to_string()));
        }
        LocalProvider::entry_from_path(&p)
    }

    fn is_dir(&self, path: &str) -> AppResult<bool> {
        Ok(fs::metadata(util::validate_local_path(path)?)?.is_dir())
    }

    fn read(&self, path: &str, max_bytes: u64) -> AppResult<Vec<u8>> {
        let p = util::validate_local_path(path)?;
        let meta = fs::metadata(&p)?;
        if meta.is_dir() {
            return Err(AppError::Unreadable("Bu bir klasör".into()));
        }
        let mut buf = Vec::with_capacity(meta.len().min(max_bytes) as usize);
        fs::File::open(&p)?.take(max_bytes).read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn write(&self, path: &str, data: &[u8]) -> AppResult<()> {
        let p = util::validate_local_path(path)?;
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                return Err(AppError::NotFound(parent.to_string_lossy().to_string()));
            }
        }
        fs::write(&p, data).map_err(AppError::from)
    }

    fn create_dir(&self, parent: &str, name: &str) -> AppResult<String> {
        let target = LocalProvider::join(parent, name)?;
        if target.exists() {
            return Err(AppError::AlreadyExists(
                target.to_string_lossy().to_string(),
            ));
        }
        fs::create_dir_all(&target)?;
        Ok(target.to_string_lossy().to_string())
    }

    fn create_file(&self, parent: &str, name: &str) -> AppResult<String> {
        let target = LocalProvider::join(parent, name)?;
        fs::File::create_new(&target)?;
        Ok(target.to_string_lossy().to_string())
    }

    fn remove(&self, path: &str, is_dir: bool) -> AppResult<()> {
        let p = util::validate_local_path(path)?;
        if is_dir {
            fs::remove_dir_all(&p)?;
        } else {
            fs::remove_file(&p)?;
        }
        Ok(())
    }

    fn rename(&self, path: &str, new_name: &str) -> AppResult<String> {
        let source = util::validate_local_path(path)?;
        let parent = source
            .parent()
            .ok_or_else(|| AppError::InvalidName("Kök klasör yeniden adlandırılamaz".into()))?;
        let target = LocalProvider::join(&parent.to_string_lossy(), new_name)?;
        if target.exists() {
            return Err(AppError::AlreadyExists(
                target.to_string_lossy().to_string(),
            ));
        }
        fs::rename(&source, &target)?;
        Ok(target.to_string_lossy().to_string())
    }

    fn copy(
        &self,
        source: &str,
        dest_dir: &str,
        dest_name: &str,
        cancel: &CancelToken,
        progress: &ProgressSink,
        transferred: &mut u64,
    ) -> AppResult<String> {
        let from = util::validate_local_path(source)?;
        let to = LocalProvider::join(dest_dir, dest_name)?;
        if to.exists() {
            return Err(AppError::AlreadyExists(to.to_string_lossy().to_string()));
        }
        copy_tree(&from, &to, cancel, progress, transferred)?;
        Ok(to.to_string_lossy().to_string())
    }

    fn move_entry(
        &self,
        source: &str,
        dest_dir: &str,
        dest_name: &str,
        cancel: &CancelToken,
        progress: &ProgressSink,
        transferred: &mut u64,
    ) -> AppResult<String> {
        let from = util::validate_local_path(source)?;
        let to = LocalProvider::join(dest_dir, dest_name)?;
        if to.exists() {
            return Err(AppError::AlreadyExists(to.to_string_lossy().to_string()));
        }
        match fs::rename(&from, &to) {
            Ok(()) => {
                if let Ok(meta) = fs::metadata(&to) {
                    if meta.is_file() {
                        *transferred += meta.len();
                    }
                }
                Ok(to.to_string_lossy().to_string())
            }
            // Farklı birim: kopyala, sonra kaynağı sil.
            Err(err) if err.raw_os_error() == Some(EXDEV) => {
                let path = self.copy(source, dest_dir, dest_name, cancel, progress, transferred)?;
                self.remove(source, from.is_dir())?;
                Ok(path)
            }
            Err(err) => Err(AppError::from(err)),
        }
    }

    fn storage_info(&self, path: &str) -> AppResult<StorageInfo> {
        // Masaüstünde taşınabilir bir API yok; uygulama Android'e özel.
        let _ = path;
        Err(AppError::Unsupported("depolama bilgisi".into()))
    }
}

/// Klasör ağacını kopyalar; iptalde yarım kalan hedefi temizler.
fn copy_tree(
    source: &Path,
    target: &Path,
    cancel: &CancelToken,
    progress: &ProgressSink,
    transferred: &mut u64,
) -> AppResult<()> {
    cancel.guard()?;
    let meta = fs::metadata(source)?;
    if meta.is_dir() {
        fs::create_dir_all(target)?;
        for item in fs::read_dir(source)? {
            let item = item?;
            let name = item.file_name();
            copy_tree(
                &item.path(),
                &target.join(&name),
                cancel,
                progress,
                transferred,
            )
            .inspect_err(|_| {
                // Yarım kalan ağacı bırakma.
                let _ = fs::remove_dir_all(target);
            })?;
        }
        return Ok(());
    }
    copy_file(source, target, cancel, progress, transferred)
}

pub fn copy_file(
    source: &Path,
    target: &Path,
    cancel: &CancelToken,
    progress: &ProgressSink,
    transferred: &mut u64,
) -> AppResult<()> {
    let mut reader = fs::File::open(source)?;
    let mut writer = fs::File::create(target)?;
    let mut buf = vec![0u8; CHUNK];
    loop {
        cancel.guard().inspect_err(|_| {
            let _ = fs::remove_file(target);
        })?;
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        std::io::Write::write_all(&mut writer, &buf[..read])?;
        *transferred += read as u64;
        progress.add(read as u64);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("b.txt"), b"merhaba").unwrap();
        fs::write(dir.path().join("a.txt"), b"dunya").unwrap();
        fs::create_dir(dir.path().join("alt")).unwrap();
        fs::write(dir.path().join("alt/ic.txt"), b"x").unwrap();
        dir
    }

    fn sink() -> ProgressSink {
        ProgressSink::silent()
    }

    #[test]
    fn listeleme_ve_stat() {
        let dir = setup();
        let provider = LocalProvider;
        let entries = provider.list(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 3);
        let file = entries.iter().find(|e| e.name == "a.txt").unwrap();
        assert_eq!(file.size, 5);
        assert_eq!(file.category, crate::models::FileCategory::Text);
        assert!(provider
            .is_dir(dir.path().join("alt").to_str().unwrap())
            .unwrap());
    }

    #[test]
    fn yaz_oku_yeniden_adlandir_sil() {
        let dir = setup();
        let provider = LocalProvider;
        let created = provider
            .create_file(dir.path().to_str().unwrap(), "yeni.txt")
            .unwrap();
        provider.write(&created, b"icerik").unwrap();
        assert_eq!(provider.read(&created, 1024).unwrap(), b"icerik");

        let renamed = provider.rename(&created, "ad.txt").unwrap();
        assert!(renamed.ends_with("ad.txt"));
        assert!(!Path::new(&created).exists());

        provider.remove(&renamed, false).unwrap();
        assert!(!Path::new(&renamed).exists());
    }

    #[test]
    fn kopyalama_alt_klasorleri_gezer() {
        let dir = setup();
        let provider = LocalProvider;
        let cancel = CancelToken::default();
        let progress = sink();
        let mut transferred = 0;
        let new_path = provider
            .copy(
                dir.path().join("alt").to_str().unwrap(),
                dir.path().to_str().unwrap(),
                "kopya",
                &cancel,
                &progress,
                &mut transferred,
            )
            .unwrap();
        assert!(Path::new(&new_path).join("ic.txt").exists());
        assert_eq!(transferred, 1);
    }

    #[test]
    fn tasima_sonrasi_kaynak_yok() {
        let dir = setup();
        let provider = LocalProvider;
        let cancel = CancelToken::default();
        let progress = sink();
        let mut transferred = 0;
        let moved = provider
            .move_entry(
                dir.path().join("a.txt").to_str().unwrap(),
                dir.path().join("alt").to_str().unwrap(),
                "a.txt",
                &cancel,
                &progress,
                &mut transferred,
            )
            .unwrap();
        assert!(Path::new(&moved).exists());
        assert!(!dir.path().join("a.txt").exists());
    }

    #[test]
    fn var_olan_ada_yazilmaz() {
        let dir = setup();
        let provider = LocalProvider;
        let err = provider
            .create_dir(dir.path().to_str().unwrap(), "alt")
            .unwrap_err();
        assert_eq!(err.code(), "ALREADY_EXISTS");
    }

    #[test]
    fn buyuk_dosya_limiti_uygulanir() {
        let dir = setup();
        let provider = LocalProvider;
        let path = dir.path().join("a.txt");
        let data = provider.read(path.to_str().unwrap(), 2).unwrap();
        assert_eq!(data, b"du");
    }

    #[test]
    fn iptal_edilen_kopyalama_temizlenir() {
        let dir = setup();
        let provider = LocalProvider;
        let cancel = CancelToken::default();
        cancel.cancel();
        let progress = sink();
        let mut transferred = 0;
        let err = provider
            .copy(
                dir.path().join("a.txt").to_str().unwrap(),
                dir.path().to_str().unwrap(),
                "kopya.txt",
                &cancel,
                &progress,
                &mut transferred,
            )
            .unwrap_err();
        assert_eq!(err.code(), "CANCELLED");
        assert!(!dir.path().join("kopya.txt").exists());
    }
}
