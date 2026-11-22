// SPDX-License-Identifier: GPL-3.0-or-later
//! Storage Access Framework sağlayıcısı. Tüm gerçek iş Kotlin tarafındaki
//! `AndroidFsPlugin` üzerinden `DocumentsContract` ile yapılır; Rust yalnızca
//! köprüyü sarar. Bu sayede kalıcı URI izinleri Android tarafında tutulur.

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::mobile;
use crate::models::{FileEntry, StorageInfo};
use crate::provider::{CancelToken, ProgressSink, StorageProvider};
use crate::util;

pub struct SafProvider<'a> {
    app: &'a AppHandle,
}

impl<'a> SafProvider<'a> {
    pub fn new(app: &'a AppHandle) -> AppResult<Self> {
        if !mobile::is_available(app) {
            return Err(AppError::AndroidOnly);
        }
        Ok(Self { app })
    }

    fn cache_file(&self, name: &str) -> AppResult<PathBuf> {
        let dir = self
            .app
            .path()
            .app_cache_dir()
            .map_err(|e| AppError::Internal(format!("önbellek klasörü: {e}")))?
            .join("saf");
        fs::create_dir_all(&dir)?;
        Ok(dir.join(name))
    }

    /// SAF belgesini geçici bir dosyaya indirir (okuma, önizleme, küçük resim).
    pub fn export_to_cache(&self, uri: &str) -> AppResult<PathBuf> {
        let name = util::display_name(uri);
        let key = format!("{:x}.bin", md5_like(uri.as_bytes()));
        let target = self.cache_file(&format!("{key}-{}", sanitize(&name)))?;
        let size = fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        let stat = self.stat(uri)?;
        if size == stat.size && size > 0 {
            return Ok(target);
        }
        let payload = json!({ "uri": uri, "destPath": target.to_string_lossy() });
        let result: mobile::ExportResult = mobile::invoke(self.app, "exportTo", payload)?;
        if result.size == 0 && stat.size > 0 {
            return Err(AppError::Unreadable(name));
        }
        Ok(target)
    }

    /// Hazır bir yerel dosyayı SAF belgesine kopyalar (bellekte tutmadan).
    pub fn import_file(&self, source: &std::path::Path, dest_uri: &str) -> AppResult<u64> {
        let payload = json!({ "srcPath": source.to_string_lossy(), "destUri": dest_uri });
        let result: mobile::ExportResult = mobile::invoke(self.app, "importFrom", payload)?;
        Ok(result.size)
    }

    fn entry_from_shared(shared: mobile::SharedEntry) -> FileEntry {
        let mut entry = FileEntry::build(
            shared.uri,
            shared.name,
            shared.is_dir,
            shared.size,
            shared.modified,
        );
        entry.read_only = shared.read_only;
        entry
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' {
                c
            } else {
                '_'
            }
        })
        .take(48)
        .collect()
}

/// Önbellek anahtarı için küçük ve bağımsız karma (SHA-256 alt kümesi).
fn md5_like(data: &[u8]) -> u64 {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(data);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes)
}

impl StorageProvider for SafProvider<'_> {
    fn list(&self, dir: &str) -> AppResult<Vec<FileEntry>> {
        let response: mobile::EntriesResponse =
            mobile::invoke(self.app, "listChildren", json!({ "uri": dir }))?;
        Ok(response
            .entries
            .into_iter()
            .map(Self::entry_from_shared)
            .collect())
    }

    fn stat(&self, path: &str) -> AppResult<FileEntry> {
        let shared: mobile::SharedEntry = mobile::invoke(self.app, "stat", json!({ "uri": path }))?;
        Ok(Self::entry_from_shared(shared))
    }

    fn is_dir(&self, path: &str) -> AppResult<bool> {
        Ok(self.stat(path)?.is_dir)
    }

    fn read(&self, path: &str, max_bytes: u64) -> AppResult<Vec<u8>> {
        let cached = self.export_to_cache(path)?;
        let mut data = fs::read(&cached)?;
        data.truncate(max_bytes.min(data.len() as u64) as usize);
        Ok(data)
    }

    fn write(&self, path: &str, data: &[u8]) -> AppResult<()> {
        let temp = self.cache_file(&format!("import-{}.tmp", md5_like(path.as_bytes())))?;
        fs::write(&temp, data)?;
        let result = self.import_file(&temp, path);
        let _ = fs::remove_file(&temp);
        result.map(|_| ())
    }

    fn create_dir(&self, parent: &str, name: &str) -> AppResult<String> {
        let result: mobile::UriResult = mobile::invoke(
            self.app,
            "createDirectory",
            json!({ "parentUri": parent, "name": name }),
        )?;
        Ok(result.uri)
    }

    fn create_file(&self, parent: &str, name: &str) -> AppResult<String> {
        let mime = util::mime_for(name, false);
        let result: mobile::UriResult = mobile::invoke(
            self.app,
            "createFile",
            json!({ "parentUri": parent, "name": name, "mime": mime }),
        )?;
        Ok(result.uri)
    }

    fn remove(&self, path: &str, _is_dir: bool) -> AppResult<()> {
        let _: serde_json::Value =
            mobile::invoke(self.app, "deleteDocument", json!({ "uri": path }))?;
        Ok(())
    }

    fn rename(&self, path: &str, new_name: &str) -> AppResult<String> {
        let result: mobile::UriResult = mobile::invoke(
            self.app,
            "renameDocument",
            json!({ "uri": path, "newName": new_name }),
        )?;
        Ok(result.uri)
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
        cancel.guard()?;
        if util::is_content_uri(source) && !self.is_dir(source)? {
            let result: mobile::UriResult = mobile::invoke(
                self.app,
                "copyDocument",
                json!({ "sourceUri": source, "destParentUri": dest_dir, "destName": dest_name }),
            )?;
            if let Ok(stat) = self.stat(&result.uri) {
                *transferred += stat.size;
                progress.add(stat.size);
            }
            return Ok(result.uri);
        }
        // Klasörler: ağacı Rust tarafında gezip her öğeyi tek tek kopyala.
        let entry = self.stat(source)?;
        let new_dir = if entry.is_dir {
            self.create_dir(dest_dir, dest_name)?
        } else {
            self.create_file(dest_dir, dest_name)?
        };
        if entry.is_dir {
            for child in self.list(source)? {
                self.copy(
                    &child.path,
                    &new_dir,
                    &child.name,
                    cancel,
                    progress,
                    transferred,
                )?;
            }
        } else {
            let data = self.read(source, u64::MAX)?;
            self.write(&new_dir, &data)?;
            *transferred += data.len() as u64;
            progress.add(data.len() as u64);
        }
        Ok(new_dir)
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
        cancel.guard()?;
        if util::is_content_uri(source) {
            let result: mobile::UriResult = mobile::invoke(
                self.app,
                "moveDocument",
                json!({ "sourceUri": source, "destParentUri": dest_dir, "destName": dest_name }),
            )?;
            return Ok(result.uri);
        }
        // Yerel kaynaktan SAF hedefine taşıma: kopyala + sil.
        let path = self.copy(source, dest_dir, dest_name, cancel, progress, transferred)?;
        fs::remove_file(source)?;
        Ok(path)
    }

    fn storage_info(&self, path: &str) -> AppResult<StorageInfo> {
        let usage: mobile::StorageUsage =
            mobile::invoke(self.app, "storageInfo", json!({ "path": path }))?;
        Ok(StorageInfo {
            total: usage.total,
            free: usage.free,
            used: usage.total.saturating_sub(usage.free),
        })
    }
}
