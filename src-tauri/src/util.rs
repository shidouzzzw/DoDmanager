// SPDX-License-Identifier: GPL-3.0-or-later
//! Yol, ad ve dosya türü yardımcıları. SAF URI'leri ile mutlak yolların
//! farkları burada tek yerde toplanır.

use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AppError, AppResult};
use crate::models::FileCategory;

pub const CONTENT_SCHEME: &str = "content://";

pub fn is_content_uri(path: &str) -> bool {
    path.starts_with(CONTENT_SCHEME)
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

pub fn system_time_to_secs(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// Yerel bir yolun son bileşeni; URI'ler için sorgu dizesi atılır.
pub fn display_name(path: &str) -> String {
    if is_content_uri(path) {
        // content://.../document/primary%3ADCIM%2Ffoto.jpg
        let trimmed = path.trim_end_matches('/');
        let tail = trimmed.rsplit('/').next().unwrap_or(trimmed);
        if let Some(pos) = tail.rfind("%2F").or_else(|| tail.rfind("%2f")) {
            return tail[pos + 3..].to_string();
        }
        return tail.to_string();
    }
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Üst klasör; kök için `None`.
pub fn parent_path(path: &str) -> Option<String> {
    if is_content_uri(path) {
        // SAF ağacında kök URI `.../tree/primary%3A` biçimindedir; üst yok.
        let without_trailing = path.trim_end_matches('/');
        if without_trailing.contains("/tree/") && without_trailing.ends_with("%3A") {
            return None;
        }
        let (base, last) = without_trailing.rsplit_once('/')?;
        if last.is_empty() {
            return None;
        }
        // `.../document/primary%3ADCIM%2Ffoto.jpg` -> `.../document/primary%3ADCIM`
        if let Some(pos) = last.rfind("%2F").or_else(|| last.rfind("%2f")) {
            return Some(format!("{base}/{}", &last[..pos]));
        }
        return None;
    }
    let parent = Path::new(path).parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    Some(parent.to_string_lossy().to_string())
}

pub fn join_path(dir: &str, name: &str) -> String {
    if is_content_uri(dir) {
        let encoded = name.replace('/', "%2F");
        format!("{}/{}", dir.trim_end_matches('/'), encoded)
    } else {
        let p = Path::new(dir).join(name);
        p.to_string_lossy().to_string()
    }
}

/// Yerel yolları doğrular: mutlak olmalı ve `..` bileşeni içermemeli.
pub fn validate_local_path(path: &str) -> AppResult<PathBuf> {
    if is_content_uri(path) {
        return Err(AppError::Internal(
            "URI yerel yol olarak kullanılamaz".into(),
        ));
    }
    let raw = Path::new(path);
    if !raw.is_absolute() {
        return Err(AppError::InvalidName(format!("Mutlak yol gerekli: {path}")));
    }
    if raw.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(AppError::InvalidName("Yolda '..' olamaz".into()));
    }
    Ok(raw.to_path_buf())
}

/// Dosya adı doğrulaması (yeniden adlandırma, yeni klasör, zip çıktısı).
pub fn validate_name(name: &str) -> AppResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidName("Ad boş olamaz".into()));
    }
    if trimmed.len() > 255 {
        return Err(AppError::InvalidName(
            "Ad 255 karakterden uzun olamaz".into(),
        ));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(AppError::InvalidName("Ad '.' veya '..' olamaz".into()));
    }
    if trimmed.contains('/') || trimmed.contains('\0') || trimmed.contains('\\') {
        return Err(AppError::InvalidName(
            "Ad eğik çizgi ya da ters bölü içeremez".into(),
        ));
    }
    Ok(())
}

/// Var olan addan çakışmasız bir ad üretir: `foto.jpg` -> `foto (1).jpg`.
pub fn unique_name(dir_paths: &[String], name: &str) -> String {
    let taken: Vec<String> = dir_paths
        .iter()
        .filter_map(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_lowercase())
        .collect();
    if !taken.iter().any(|t| t == &name.to_lowercase()) {
        return name.to_string();
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for i in 1..10_000 {
        let candidate = format!("{stem} ({i}){ext}");
        if !taken.iter().any(|t| t == &candidate.to_lowercase()) {
            return candidate;
        }
    }
    format!("{stem}-{}{ext}", now_secs())
}

pub fn mime_for(name: &str, is_dir: bool) -> String {
    if is_dir {
        return "inode/directory".into();
    }
    let lower = name.to_lowercase();
    let known = match lower.rsplit('.').next() {
        Some("md") | Some("markdown") => Some("text/markdown"),
        Some("rs") => Some("text/rust"),
        Some("ts") | Some("tsx") => Some("text/typescript"),
        Some("js") | Some("jsx") | Some("mjs") => Some("text/javascript"),
        Some("kt") | Some("kts") => Some("text/kotlin"),
        Some("json") => Some("application/json"),
        Some("toml") => Some("text/toml"),
        Some("yml") | Some("yaml") => Some("text/yaml"),
        Some("csv") => Some("text/csv"),
        Some("log") => Some("text/plain"),
        Some("7z") => Some("application/x-7z-compressed"),
        Some("tar") => Some("application/x-tar"),
        Some("rar") => Some("application/vnd.rar"),
        _ => None,
    };
    if let Some(mime) = known {
        return mime.to_string();
    }
    mime_guess::from_path(name)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}

pub fn category_for(name: &str, is_dir: bool) -> FileCategory {
    if is_dir {
        return FileCategory::Folder;
    }
    let ext = name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let category = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "heic" | "heif" | "avif" | "svg"
        | "tif" | "tiff" => FileCategory::Image,
        "mp4" | "mkv" | "webm" | "mov" | "avi" | "3gp" | "m4v" => FileCategory::Video,
        "mp3" | "m4a" | "aac" | "flac" | "ogg" | "opus" | "wav" | "amr" | "mid" => {
            FileCategory::Audio
        }
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        | "epub" => FileCategory::Document,
        "zip" | "apk" | "jar" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" => {
            FileCategory::Archive
        }
        "txt" | "md" | "markdown" | "json" | "xml" | "html" | "css" | "js" | "ts" | "tsx"
        | "jsx" | "rs" | "kt" | "kts" | "java" | "py" | "rb" | "go" | "c" | "h" | "cpp" | "hpp"
        | "sh" | "yml" | "yaml" | "toml" | "ini" | "conf" | "csv" | "log" | "sql" => {
            FileCategory::Text
        }
        _ => {
            if mime_for(name, false).starts_with("text/") {
                FileCategory::Text
            } else {
                FileCategory::Other
            }
        }
    };
    category
}

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

/// İnsan okur biçimde boyut (arayüz de biçimlendirir; log ve test için).
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileCategory;

    #[test]
    fn ad_ve_uzanti_turetilir() {
        assert_eq!(
            display_name("/storage/emulated/0/DCIM/foto.jpg"),
            "foto.jpg"
        );
        assert_eq!(category_for("foto.JPG", false), FileCategory::Image);
        assert_eq!(category_for("arsiv.zip", false), FileCategory::Archive);
        assert_eq!(category_for("Klasor", true), FileCategory::Folder);
        assert_eq!(mime_for("notlar.md", false), "text/markdown");
        assert_eq!(
            mime_for("bilinmeyen.zzz", false),
            "application/octet-stream"
        );
    }

    #[test]
    fn ust_klasor_ve_kok() {
        assert_eq!(
            parent_path("/storage/emulated/0/DCIM/foto.jpg").as_deref(),
            Some("/storage/emulated/0/DCIM")
        );
        assert_eq!(parent_path("/"), None);
    }

    #[test]
    fn guvenli_olmayan_yollar_reddedilir() {
        assert!(validate_local_path("/storage/../../etc/passwd").is_err());
        assert!(validate_local_path("goreli/yol").is_err());
        assert!(validate_local_path("/storage/emulated/0").is_ok());
    }

    #[test]
    fn ad_dogrulama_ve_benzersiz_ad() {
        assert!(validate_name("rapor.pdf").is_ok());
        assert!(validate_name("a/b").is_err());
        assert!(validate_name("  ").is_err());
        let existing = vec!["/x/foto.jpg".to_string(), "/x/foto (1).jpg".to_string()];
        assert_eq!(unique_name(&existing, "foto.jpg"), "foto (2).jpg");
        assert_eq!(unique_name(&existing, "baska.jpg"), "baska.jpg");
    }

    #[test]
    fn boyut_bicimlendirme() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn uri_yol_parcalari() {
        // SAF URI'lerinde üst klasör belge kimliğiyle çözülür; path kırpma yanlıştır.
        let uri = "content://com.android.externalstorage.documents/tree/primary%3ADCIM";
        assert_eq!(parent_path(uri), None);
        assert_eq!(join_path("/a", "b.txt"), "/a/b.txt");
    }
}
