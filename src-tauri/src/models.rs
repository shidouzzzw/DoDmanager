// SPDX-License-Identifier: GPL-3.0-or-later
//! Komutların ve arayüzün paylaştığı veri tipleri.

use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Text,
    Folder,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Sağlayıcıya özgü tanıtıcı: mutlak yol ya da `content://` URI.
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    /// Unix zaman damgası (saniye).
    pub modified: i64,
    pub mime: String,
    pub category: FileCategory,
    pub read_only: bool,
    pub child_count: Option<u64>,
}

impl FileEntry {
    pub fn build(path: String, name: String, is_dir: bool, size: u64, modified: i64) -> Self {
        Self {
            mime: util::mime_for(&name, is_dir),
            category: util::category_for(&name, is_dir),
            read_only: false,
            child_count: None,
            path,
            name,
            is_dir,
            size,
            modified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crumb {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub breadcrumb: Vec<Crumb>,
    pub is_root: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessMode {
    #[default]
    Scoped,
    AllFiles,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortKey {
    #[default]
    Name,
    Date,
    Size,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub locale: String,
    pub theme: ThemeMode,
    pub accent: String,
    pub dynamic_color: bool,
    pub view: ViewMode,
    pub sort: SortKey,
    pub sort_desc: bool,
    pub haptics: bool,
    pub show_hidden: bool,
    pub reduced_motion: bool,
    pub grid_columns: u8,
    pub access_mode: AccessMode,
    pub onboarding_complete: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: "tr".into(),
            theme: ThemeMode::System,
            accent: "indigo".into(),
            dynamic_color: false,
            view: ViewMode::Grid,
            sort: SortKey::Name,
            sort_desc: false,
            haptics: true,
            show_hidden: false,
            reduced_motion: false,
            grid_columns: 3,
            access_mode: AccessMode::Scoped,
            onboarding_complete: false,
        }
    }
}

/// Kullanıcının eklediği bir konum (SAF ağacı ya da mutlak yol).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: String,
    pub label: String,
    /// `content://` URI ya da mutlak yol.
    pub path: String,
    pub provider: String,
    pub removable: bool,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessStatus {
    pub mode: AccessMode,
    pub has_all_files_access: bool,
    pub scoped_locations: u32,
    pub default_location: Option<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub job_id: String,
    pub kind: String,
    pub done: u64,
    pub total: u64,
    pub current: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobFinished {
    pub job_id: String,
    pub kind: String,
    pub ok: bool,
    pub message: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub entry: FileEntry,
    pub parent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDone {
    pub search_id: String,
    pub scanned: u64,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub id: String,
    pub name: String,
    pub original_path: String,
    /// Geri yükleme hedefi: silme anındaki üst klasör.
    pub original_parent: String,
    pub is_dir: bool,
    pub size: u64,
    pub deleted_at: i64,
    pub location_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileResult {
    /// Metin dosyaları için UTF-8 içerik (en fazla `limit` bayt).
    pub text: Option<String>,
    /// İkili dosyalar için base64.
    pub base64: Option<String>,
    pub size: u64,
    pub mime: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub identifier: String,
    pub platform: String,
    pub bridge_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipResult {
    pub output: String,
    pub entries: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferMode {
    Copy,
    Move,
}

/// Favoriler ve son kullanılanlar için hafif kayıt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPath {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub mime: String,
    pub touched_at: i64,
    #[serde(default)]
    pub location_id: Option<String>,
}

impl SavedPath {
    pub fn from_entry(entry: &FileEntry, location_id: Option<String>) -> Self {
        Self {
            path: entry.path.clone(),
            name: entry.name.clone(),
            is_dir: entry.is_dir,
            mime: entry.mime.clone(),
            touched_at: util::now_secs(),
            location_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbResult {
    /// Önbellekteki görselin mutlak yolu; arayüz `convertFileSrc` uygular.
    pub path: String,
    pub cached: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAsset {
    /// Önizleme için önbelleğe alınmış dosyanın mutlak yolu.
    pub path: String,
    /// Özgün dosyanın sağlayıcı yolculuğu.
    pub source: String,
    pub mime: String,
    pub size: u64,
    pub kind: String,
}
