// SPDX-License-Identifier: GPL-3.0-or-later
//! ZIP oluşturma ve açma. Girişler akışlı yazılır, ilerleme bayt bazında
//! bildirilir. Açma sırasında yol kaçışı (`../`) ve zip bombası engellenir.

use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::AppHandle;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{AppError, AppResult};
use crate::jobs::JobCtx;
use crate::models::ZipResult;
use crate::provider::{self, saf::SafProvider};
use crate::transfer;
use crate::util;

const MAX_ENTRIES: usize = 100_000;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;

/// Kaynakları tek bir ZIP'te toplar ve hedef klasöre bırakır.
pub fn zip_paths(
    ctx: &Arc<JobCtx>,
    sources: Vec<String>,
    dest_dir: String,
    archive_name: String,
) -> AppResult<ZipResult> {
    let app = ctx.app_handle();
    let name = if archive_name.to_lowercase().ends_with(".zip") {
        archive_name
    } else {
        format!("{archive_name}.zip")
    };
    util::validate_name(&name)?;
    let dest_name = provider::unique_name_in(&app, &dest_dir, &name)?;

    let temp_dir = transfer::cache_dir(&app, "zips")?;
    let temp = temp_dir.join(format!("{}.zip", uuid::Uuid::new_v4()));
    let file = fs::File::create(&temp)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut total = 0u64;
    for source in &sources {
        total += measure(&app, source).unwrap_or(0);
    }
    ctx.set_total(total.max(1));
    let progress = ctx.sink();
    let mut entries = 0u64;
    let mut bytes = 0u64;

    for source in &sources {
        ctx.guard()?;
        let entry = provider::stat(&app, source)?;
        add_entry(
            &app,
            ctx,
            &mut zip,
            &entry.path,
            &entry.name,
            options,
            &progress,
            &mut entries,
            &mut bytes,
        )?;
    }
    zip.finish()?;

    let output =
        transfer::place_local_file(&app, &temp, &dest_dir, &dest_name, &ctx.cancel, &progress)?;
    let _ = fs::remove_file(&temp);

    Ok(ZipResult {
        output,
        entries,
        bytes,
    })
}

fn measure(app: &AppHandle, path: &str) -> AppResult<u64> {
    let entry = provider::stat(app, path)?;
    if !entry.is_dir {
        return Ok(entry.size);
    }
    let mut total = 0;
    for child in provider::list(app, path)? {
        total += measure(app, &child.path).unwrap_or(0);
    }
    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn add_entry(
    app: &AppHandle,
    ctx: &Arc<JobCtx>,
    zip: &mut ZipWriter<fs::File>,
    path: &str,
    name_in_zip: &str,
    options: SimpleFileOptions,
    progress: &crate::provider::ProgressSink,
    entries: &mut u64,
    bytes: &mut u64,
) -> AppResult<()> {
    ctx.guard()?;
    ctx.set_current(name_in_zip);
    let entry = provider::stat(app, path)?;
    if entry.is_dir {
        zip.add_directory(format!("{name_in_zip}/"), options)?;
        for child in provider::list(app, path)? {
            add_entry(
                app,
                ctx,
                zip,
                &child.path,
                &format!("{name_in_zip}/{}", child.name),
                options,
                progress,
                entries,
                bytes,
            )?;
        }
        return Ok(());
    }
    if *entries as usize >= MAX_ENTRIES {
        return Err(AppError::Archive("Arşivde çok fazla öğe var".into()));
    }
    zip.start_file(name_in_zip.to_string(), options)?;
    let mut reader = EntryReader::open(app, path)?;
    let mut buf = vec![0u8; 128 * 1024];
    loop {
        ctx.guard()?;
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        zip.write_all(&buf[..read])?;
        *bytes += read as u64;
        progress.add(read as u64);
    }
    *entries += 1;
    Ok(())
}

/// Yerel dosya ya da SAF belgesi için akışlı okuyucu.
enum EntryReader {
    File(fs::File),
    Cached { file: fs::File, temp: PathBuf },
}

impl EntryReader {
    fn open(app: &AppHandle, path: &str) -> AppResult<Self> {
        if util::is_content_uri(path) {
            let temp = SafProvider::new(app)?.export_to_cache(path)?;
            let file = fs::File::open(&temp)?;
            Ok(Self::Cached { file, temp })
        } else {
            Ok(Self::File(fs::File::open(path)?))
        }
    }
}

impl Read for EntryReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::File(file) => file.read(buf),
            Self::Cached { file, .. } => file.read(buf),
        }
    }
}

impl Drop for EntryReader {
    fn drop(&mut self) {
        if let Self::Cached { temp, .. } = self {
            let _ = fs::remove_file(temp);
        }
    }
}

/// Arşivi hedef klasöre açar.
pub fn unzip(ctx: &Arc<JobCtx>, archive: String, dest_dir: String) -> AppResult<ZipResult> {
    let app = ctx.app_handle();
    let (archive_path, temp_guard) = if util::is_content_uri(&archive) {
        let temp = SafProvider::new(&app)?.export_to_cache(&archive)?;
        (temp.clone(), Some(temp))
    } else {
        (PathBuf::from(&archive), None)
    };

    let file = fs::File::open(&archive_path)?;
    let mut zip = ZipArchive::new(file)?;
    let total: u64 = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|f| f.size()))
        .sum();
    ctx.set_total(total.max(1));
    let progress = ctx.sink();

    let mut extracted = 0u64;
    let mut bytes = 0u64;
    let mut dirs: Vec<String> = Vec::new();

    for index in 0..zip.len() {
        ctx.guard()?;
        let mut entry = zip.by_index(index)?;
        let Some(safe_name) = entry.enclosed_name() else {
            return Err(AppError::Archive("Arşivde geçersiz yol var".into()));
        };
        let name = safe_name.to_string_lossy().replace('\\', "/");
        if name.is_empty() {
            continue;
        }
        if entry.size() > 0 && bytes.saturating_add(entry.size()) > MAX_TOTAL_BYTES {
            return Err(AppError::Archive("Arşiv açılımı çok büyük".into()));
        }
        ctx.set_current(&name);

        // Üst klasörleri sağlayıcı üzerinden oluştur.
        let mut current_dir = dest_dir.clone();
        let segments: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
        let (file_name, parents) = match segments.split_last() {
            Some((last, parents)) => (*last, parents),
            None => continue,
        };
        for segment in parents {
            let candidate = format!("{current_dir}/{segment}");
            if !dirs.contains(&candidate) {
                if !provider::is_dir(&app, &candidate).unwrap_or(false) {
                    provider::create_dir(&app, &current_dir, segment)?;
                    dirs.push(candidate.clone());
                } else {
                    dirs.push(candidate.clone());
                }
            }
            current_dir = candidate;
        }

        if entry.is_dir() {
            if !provider::is_dir(&app, &format!("{current_dir}/{file_name}")).unwrap_or(false) {
                provider::create_dir(&app, &current_dir, file_name)?;
            }
            continue;
        }

        // Girişi önce önbellek dosyasına çıkar, sonra hedefe akıt.
        let staging = transfer::cache_dir(&app, "unzip")?;
        let staged = staging.join(format!("{}.part", uuid::Uuid::new_v4()));
        {
            let mut out = fs::File::create(&staged)?;
            let mut buf = vec![0u8; 128 * 1024];
            loop {
                ctx.guard()?;
                let read = entry.read(&mut buf)?;
                if read == 0 {
                    break;
                }
                out.write_all(&buf[..read])?;
                progress.add(read as u64);
            }
        }
        let placed = transfer::place_local_file(
            &app,
            &staged,
            &current_dir,
            file_name,
            &ctx.cancel,
            &progress,
        )?;
        let _ = fs::remove_file(&staged);
        let _ = placed;
        extracted += 1;
        bytes += entry.size();
    }

    ctx.finish_ok();
    if let Some(temp) = temp_guard {
        let _ = fs::remove_file(temp);
    }
    Ok(ZipResult {
        output: dest_dir,
        entries: extracted,
        bytes,
    })
}

/// ZIP içerik listesini döndürür (önizleme ve hızlı bakış için).
pub fn list_zip(app: &AppHandle, archive: &str) -> AppResult<Vec<String>> {
    let path = if util::is_content_uri(archive) {
        SafProvider::new(app)?.export_to_cache(archive)?
    } else {
        PathBuf::from(archive)
    };
    let file = fs::File::open(&path)?;
    let mut zip = ZipArchive::new(file)?;
    let mut names = Vec::new();
    for index in 0..zip.len() {
        if let Ok(entry) = zip.by_index(index) {
            names.push(entry.name().to_string());
        }
    }
    Ok(names)
}

/// Arşivin diskteki boyutu (ilerleme çubuğu için üst sınır).
pub fn archive_size(app: &AppHandle, archive: &str) -> AppResult<u64> {
    Ok(provider::stat(app, archive)?.size)
}

/// Açma sırasında kullanılan geçici klasörü temizler.
pub fn clean_temp(app: &AppHandle) {
    if let Ok(dir) = transfer::cache_dir(app, "unzip") {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

/// Zip girişlerini okumak için kullanılan yardımcı (testlerde).
pub fn open_zip(path: &Path) -> AppResult<ZipArchive<impl Read + Seek>> {
    Ok(ZipArchive::new(fs::File::open(path)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_yol_kacisi_engellenir() {
        // `enclosed_name` yalnızca güvenli adları kabul eder.
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("kotu.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut writer = ZipWriter::new(file);
            let options = SimpleFileOptions::default();
            writer.start_file("../kacis.txt", options).unwrap();
            writer.write_all(b"x").unwrap();
            writer.finish().unwrap();
        }
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let entry = archive.by_index(0).unwrap();
        assert!(entry.enclosed_name().is_none());
    }

    #[test]
    fn zip_olustur_ve_oku() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut writer = ZipWriter::new(file);
            writer
                .start_file("a.txt", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"merhaba").unwrap();
            writer.finish().unwrap();
        }
        let mut archive = open_zip(&zip_path).unwrap();
        let mut entry = archive.by_index(0).unwrap();
        let mut text = String::new();
        entry.read_to_string(&mut text).unwrap();
        assert_eq!(text, "merhaba");
    }
}
