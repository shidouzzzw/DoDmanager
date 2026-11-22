// SPDX-License-Identifier: GPL-3.0-or-later
//! Küçük resim üretimi. Görseller `image` ile çözülür, en fazla `2 * px`
//! boyutuna indirilir ve diske JPEG olarak önbelleğe yazılır. Liste bu komutu
//! görünür öğeler için tembel çağırır.

use std::fs;
use std::path::{Path, PathBuf};

use image::ImageReader;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::models::{FileCategory, ThumbResult};
use crate::provider::{self, saf::SafProvider};
use crate::transfer;
use crate::util;

/// Bellek koruması: aşırı büyük görseller reddedilir.
const MAX_DIMENSION: u32 = 16_384;
const MAX_ALLOC: u64 = 256 * 1024 * 1024;

fn cache_key(path: &str, modified: i64, size: u64, px: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hasher.update(modified.to_le_bytes());
    hasher.update(size.to_le_bytes());
    hasher.update(px.to_le_bytes());
    let digest = hasher.finalize();
    digest[..16].iter().map(|b| format!("{b:02x}")).collect()
}

fn open_limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

/// Görsel küçük resmi üretir; desteklenmeyen türlerde `UNSUPPORTED` döner.
pub fn generate(app: &AppHandle, path: &str, px: u32) -> AppResult<ThumbResult> {
    let entry = provider::stat(app, path)?;
    if entry.is_dir || entry.category != FileCategory::Image {
        return Err(AppError::Unsupported("küçük resim".into()));
    }
    let px = px.clamp(48, 512);
    let key = cache_key(&entry.path, entry.modified, entry.size, px);
    let dir = transfer::cache_dir(app, "thumbnails")?;
    let target = dir.join(format!("{key}.jpg"));

    if target.exists() {
        let (width, height) = image::image_dimensions(&target).unwrap_or((px, px));
        return Ok(ThumbResult {
            path: target.to_string_lossy().to_string(),
            cached: true,
            width,
            height,
        });
    }

    // SAF belgeleri önce önbelleğe indirilir, sonra çözülür.
    let source_path: PathBuf = if util::is_content_uri(&entry.path) {
        SafProvider::new(app)?.export_to_cache(&entry.path)?
    } else {
        PathBuf::from(&entry.path)
    };

    let image = read_image(&source_path)?;
    let thumb = image.thumbnail(px * 2, px * 2);
    let mut output = fs::File::create(&target)?;
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 78);
    encoder.encode_image(&thumb)?;
    drop(output);

    if util::is_content_uri(&entry.path)
        && source_path.starts_with(transfer::cache_dir(app, "saf")?)
    {
        let _ = fs::remove_file(&source_path);
    }

    Ok(ThumbResult {
        path: target.to_string_lossy().to_string(),
        cached: false,
        width: thumb.width(),
        height: thumb.height(),
    })
}

fn read_image(path: &Path) -> AppResult<image::DynamicImage> {
    let mut reader = ImageReader::open(path)
        .map_err(|e| AppError::Unreadable(format!("görsel açılamadı: {e}")))?
        .with_guessed_format()
        .map_err(|e| AppError::Unreadable(format!("biçim: {e}")))?;
    reader.limits(open_limits());
    let image = reader
        .decode()
        .map_err(|e| AppError::Unreadable(format!("görsel çözülemedi: {e}")))?;
    Ok(image)
}

/// Önizleme için önbelleğe kopyalanmış dosya yolu üretir.
pub fn preview_cache_path(app: &AppHandle, path: &str) -> AppResult<PathBuf> {
    let entry = provider::stat(app, path)?;
    let key = cache_key(&entry.path, entry.modified, entry.size, 0);
    let ext = Path::new(&entry.name)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .filter(|e| e.len() <= 8 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "bin".into());
    let dir = transfer::cache_dir(app, "previews")?;
    let target = dir.join(format!("{key}.{ext}"));
    if target.exists() {
        return Ok(target);
    }
    if util::is_content_uri(&entry.path) {
        let cached = SafProvider::new(app)?.export_to_cache(&entry.path)?;
        fs::copy(&cached, &target)?;
        let _ = fs::remove_file(&cached);
    } else {
        fs::copy(&entry.path, &target)?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onbellek_anahtari_ayni_girdide_ayni() {
        let a = cache_key("/a/foto.jpg", 100, 2048, 256);
        let b = cache_key("/a/foto.jpg", 100, 2048, 256);
        let c = cache_key("/a/foto.jpg", 101, 2048, 256);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 32);
    }
}
