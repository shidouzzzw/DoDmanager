// SPDX-License-Identifier: GPL-3.0-or-later
//! Kopyalama/taşıma motoru. Aynı sağlayıcı içinde doğrudan, sağlayıcılar arası
//! işlemlerde önbellek üzerinden akışlı çalışır. SAF ↔ yerel geçişlerinde dosya
//! belleğe alınmaz; önbellek dosyası üzerinden taşınır.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::jobs::JobCtx;
use crate::models::{FileEntry, TransferMode};
use crate::provider::{
    self, local::LocalProvider, saf::SafProvider, CancelToken, ProgressSink, StorageProvider,
};
use crate::util;

pub struct TransferOutcome {
    pub entry: FileEntry,
    pub source: String,
}

/// Birden fazla öğeyi hedef klasöre kopyalar/taşır.
pub fn transfer_many(
    ctx: &Arc<JobCtx>,
    sources: Vec<String>,
    dest_dir: String,
    mode: TransferMode,
) -> AppResult<Vec<TransferOutcome>> {
    let app = ctx_app(ctx);
    let dest_is_saf = util::is_content_uri(&dest_dir);

    // İlerleme için toplam boyut (klasörlerin boyutu 0 sayılır; çubuk belirsiz olur).
    let mut total = 0u64;
    for source in &sources {
        if let Ok(entry) = provider::stat(&app, source) {
            if !entry.is_dir {
                total += entry.size;
            }
        }
    }
    ctx.set_total(total);
    let progress = ctx.sink();
    let cancel = ctx.cancel.clone();
    let mut transferred = 0u64;
    let mut outcomes = Vec::new();

    // Hedef klasördeki adlar bir kez okunur; çakışmalar sırayla çözülür.
    let mut existing: Vec<String> = provider::existing_names(&app, &dest_dir)?;

    for source in sources {
        ctx.guard()?;
        let entry = provider::stat(&app, &source)?;
        let name = if entry.name.is_empty() {
            util::display_name(&source)
        } else {
            entry.name.clone()
        };
        let dest_name = util::unique_name(&existing, &name);
        existing.push(dest_name.clone());
        ctx.set_current(&dest_name);

        let new_path =
            if mode == TransferMode::Move && !dest_is_saf && !util::is_content_uri(&source) {
                LocalProvider.move_entry(
                    &source,
                    &dest_dir,
                    &dest_name,
                    &cancel,
                    &progress,
                    &mut transferred,
                )?
            } else if dest_is_saf && util::is_content_uri(&source) && mode == TransferMode::Move {
                let provider = SafProvider::new(&app)?;
                provider.move_entry(
                    &source,
                    &dest_dir,
                    &dest_name,
                    &cancel,
                    &progress,
                    &mut transferred,
                )?
            } else {
                let new_path = copy_cross(
                    &app,
                    &source,
                    &dest_dir,
                    &dest_name,
                    &cancel,
                    &progress,
                    &mut transferred,
                )?;
                if mode == TransferMode::Move {
                    provider::remove(&app, &source)?;
                }
                new_path
            };

        outcomes.push(TransferOutcome {
            entry: provider::stat(&app, &new_path)?,
            source,
        });
    }

    Ok(outcomes)
}

/// Tek bir öğeyi hedef klasöre kopyalar (çöp kutusu ve geri yükleme için).
pub fn copy_path(
    app: &AppHandle,
    source: &str,
    dest_dir: &str,
    dest_name: &str,
    cancel: &CancelToken,
    progress: &ProgressSink,
) -> AppResult<String> {
    let mut transferred = 0u64;
    copy_cross(
        app,
        source,
        dest_dir,
        dest_name,
        cancel,
        progress,
        &mut transferred,
    )
}

/// Sağlayıcılar arası kopyalama.
fn copy_cross(
    app: &AppHandle,
    source: &str,
    dest_dir: &str,
    dest_name: &str,
    cancel: &CancelToken,
    progress: &ProgressSink,
    transferred: &mut u64,
) -> AppResult<String> {
    let src_saf = util::is_content_uri(source);
    let dst_saf = util::is_content_uri(dest_dir);

    match (src_saf, dst_saf) {
        (false, false) => {
            LocalProvider.copy(source, dest_dir, dest_name, cancel, progress, transferred)
        }
        (true, true) => {
            SafProvider::new(app)?.copy(source, dest_dir, dest_name, cancel, progress, transferred)
        }
        (false, true) => {
            let saf = SafProvider::new(app)?;
            let entry = LocalProvider.stat(source)?;
            if entry.is_dir {
                let new_dir = saf.create_dir(dest_dir, dest_name)?;
                for child in LocalProvider.list(source)? {
                    copy_cross(
                        app,
                        &child.path,
                        &new_dir,
                        &child.name,
                        cancel,
                        progress,
                        transferred,
                    )?;
                }
                Ok(new_dir)
            } else {
                let new_uri = saf.create_file(dest_dir, dest_name)?;
                let size = saf.import_file(Path::new(source), &new_uri)?;
                *transferred += size;
                progress.add(size);
                Ok(new_uri)
            }
        }
        (true, false) => {
            let saf = SafProvider::new(app)?;
            let entry = saf.stat(source)?;
            if entry.is_dir {
                let new_dir = LocalProvider.create_dir(dest_dir, dest_name)?;
                for child in saf.list(source)? {
                    copy_cross(
                        app,
                        &child.path,
                        &new_dir,
                        &child.name,
                        cancel,
                        progress,
                        transferred,
                    )?;
                }
                Ok(new_dir)
            } else {
                // SAF belgesini önbelleğe indirip yerel hedefe kopyala.
                let cached = saf.export_to_cache(source)?;
                let meta = fs::metadata(&cached)?;
                let new_path = LocalProvider.copy(
                    &cached.to_string_lossy(),
                    dest_dir,
                    dest_name,
                    cancel,
                    progress,
                    transferred,
                )?;
                *transferred += meta.len();
                Ok(new_path)
            }
        }
    }
}

/// `JobCtx` içindeki uygulama tutamacı.
fn ctx_app(ctx: &Arc<JobCtx>) -> AppHandle {
    ctx.app_handle()
}

/// Yerel bir önbellek dosyasını hedef klasöre yerleştirir (zip çıktısı için).
pub fn place_local_file(
    app: &AppHandle,
    temp_file: &Path,
    dest_dir: &str,
    dest_name: &str,
    cancel: &CancelToken,
    progress: &ProgressSink,
) -> AppResult<String> {
    if util::is_content_uri(dest_dir) {
        let saf = SafProvider::new(app)?;
        let uri = saf.create_file(dest_dir, dest_name)?;
        let size = saf.import_file(temp_file, &uri)?;
        progress.add(size);
        Ok(uri)
    } else {
        let mut transferred = 0;
        LocalProvider.copy(
            &temp_file.to_string_lossy(),
            dest_dir,
            dest_name,
            cancel,
            progress,
            &mut transferred,
        )
    }
}

/// Uygulama önbelleğinde geçici klasör.
pub fn cache_dir(app: &AppHandle, sub: &str) -> AppResult<std::path::PathBuf> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Internal(format!("önbellek: {e}")))?
        .join(sub);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::local::LocalProvider;
    use std::fs;

    #[test]
    fn benzersiz_ad_hedef_klasorde_cakismaz() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"1").unwrap();
        let names = LocalProvider.list(dir.path().to_str().unwrap()).unwrap();
        let existing: Vec<String> = names.into_iter().map(|e| e.name).collect();
        let chosen = util::unique_name(&existing, "a.txt");
        assert_eq!(chosen, "a (1).txt");
    }
}
