// SPDX-License-Identifier: GPL-3.0-or-later
//! Arayüzün çağırdığı tüm Tauri komutları. Her komut `StorageProvider`
//! katmanına iner; arayüz dosya sistemine doğrudan dokunmaz.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, State};

use crate::archive;
use crate::error::{AppError, AppResult};
use crate::jobs::{JobCtx, JobRegistry};
use crate::mobile;
use crate::models::*;
use crate::provider::{self, saf::SafProvider, StorageProvider};
use crate::search::{self, SearchRegistry};
use crate::settings;
use crate::thumbnails;
use crate::transfer;
use crate::trash;
use crate::util;

const MAX_TEXT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 24 * 1024 * 1024;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub locations: Mutex<Vec<Location>>,
    pub favorites: Mutex<Vec<SavedPath>>,
    pub recents: Mutex<Vec<SavedPath>>,
    pub jobs: Arc<JobRegistry>,
    pub searches: Arc<SearchRegistry>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Self {
        let settings = settings::load_settings(app);
        let locations = settings::load_locations(app);
        let favorites = settings::load_favorites(app);
        let recents = settings::load_recents(app);
        if let Err(err) = settings::app_private_dir(app) {
            eprintln!("uygulama klasörü hazırlanamadı: {err}");
        }
        Self {
            settings: Mutex::new(settings),
            locations: Mutex::new(locations),
            favorites: Mutex::new(favorites),
            recents: Mutex::new(recents),
            jobs: Arc::new(JobRegistry::default()),
            searches: Arc::new(SearchRegistry::default()),
        }
    }

    fn settings_snapshot(&self) -> AppSettings {
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRequest {
    pub path: String,
    pub parent: String,
}

/// İşi arka plan iş parçacığında çalıştırır, event'lerle ilerleme yayınlar.
async fn run_job<T, F>(
    app: AppHandle,
    registry: Arc<JobRegistry>,
    kind: &'static str,
    job_id: String,
    work: F,
) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce(Arc<JobCtx>) -> AppResult<T> + Send + 'static,
{
    let ctx = JobCtx::start(&app, &registry, kind, &job_id);
    let worker = Arc::clone(&ctx);
    let result = tauri::async_runtime::spawn_blocking(move || work(worker))
        .await
        .map_err(|e| AppError::Internal(format!("iş parçacığı: {e}")))?;
    match &result {
        Ok(_) => ctx.finish_ok(),
        Err(err) => ctx.finish_err(err),
    }
    result
}

// ---------------------------------------------------------------------------
// Ayarlar ve erişim modeli
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings_snapshot()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> AppResult<AppSettings> {
    let mut sanitized = settings;
    if !matches!(sanitized.locale.as_str(), "tr" | "en") {
        sanitized.locale = "tr".into();
    }
    if !matches!(
        sanitized.accent.as_str(),
        "indigo" | "ink" | "moss" | "amber" | "clay" | "plum"
    ) {
        sanitized.accent = "indigo".into();
    }
    sanitized.grid_columns = sanitized.grid_columns.clamp(2, 5);
    settings::save_settings(&app, &sanitized)?;
    if let Ok(mut current) = state.settings.lock() {
        *current = sanitized.clone();
    }
    Ok(sanitized)
}

#[tauri::command]
pub fn access_status(app: AppHandle, state: State<'_, AppState>) -> AccessStatus {
    build_access_status(&app, &state)
}

fn build_access_status(app: &AppHandle, state: &AppState) -> AccessStatus {
    let settings = state.settings_snapshot();
    let granted = if mobile::is_available(app) {
        mobile::invoke::<mobile::GrantStatus>(app, "hasAllFilesAccess", json!({}))
            .map(|g| g.granted)
            .unwrap_or(false)
    } else {
        false
    };
    let scoped = state
        .locations
        .lock()
        .map(|l| {
            l.iter()
                .filter(|x| x.path.starts_with("content://"))
                .count() as u32
        })
        .unwrap_or(0);
    AccessStatus {
        mode: settings.access_mode,
        has_all_files_access: granted,
        scoped_locations: scoped,
        default_location: default_location_for(app, state),
    }
}

#[tauri::command]
pub fn set_access_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: AccessMode,
) -> AppResult<AccessStatus> {
    if mode == AccessMode::AllFiles {
        let granted = mobile::invoke::<mobile::GrantStatus>(&app, "hasAllFilesAccess", json!({}))
            .map(|g| g.granted)
            .unwrap_or(false);
        if !granted {
            return Err(AppError::PermissionDenied);
        }
    }
    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| AppError::Internal("ayar kilidi".into()))?;
        settings.access_mode = mode;
        settings::save_settings(&app, &settings)?;
    }
    Ok(build_access_status(&app, &state))
}

/// "Tüm dosyalara erişim" sistem ayarını açar; kullanıcı dönünce durumu tazelenir.
#[tauri::command]
pub fn request_all_files_access(app: AppHandle) -> AppResult<()> {
    let _: serde_json::Value = mobile::invoke(&app, "openAllFilesAccessSettings", json!({}))?;
    Ok(())
}

#[tauri::command]
pub fn open_app_settings(app: AppHandle) -> AppResult<()> {
    let _: serde_json::Value = mobile::invoke(&app, "openAppSettings", json!({}))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Konumlar ve depolama
// ---------------------------------------------------------------------------

fn saved_locations(state: &AppState) -> Vec<Location> {
    state
        .locations
        .lock()
        .map(|l| l.clone())
        .unwrap_or_default()
}

fn builtin_locations(app: &AppHandle, settings: &AppSettings) -> Vec<Location> {
    let mut list = Vec::new();
    if let Ok(private) = settings::app_private_location(app) {
        list.push(private);
    }
    if settings.access_mode == AccessMode::AllFiles {
        if let Ok(response) =
            mobile::invoke::<mobile::RootsResponse>(app, "externalRoots", json!({}))
        {
            for root in response.roots {
                list.push(Location {
                    id: format!("root-{}", root.path.replace('/', "_")),
                    label: root.label,
                    path: root.path,
                    provider: "local".into(),
                    removable: root.removable,
                    added_at: 0,
                });
            }
        } else if cfg!(not(target_os = "android")) {
            // Masaüstü geliştirme: ev klasörü sahte değil, gerçek yol.
            if let Ok(home) = std::env::var("HOME") {
                list.push(Location {
                    id: "root-home".into(),
                    label: "Ev klasörü".into(),
                    path: home,
                    provider: "local".into(),
                    removable: false,
                    added_at: 0,
                });
            }
        }
    }
    list
}

fn all_locations(app: &AppHandle, state: &AppState) -> Vec<Location> {
    let settings = state.settings_snapshot();
    let mut list = builtin_locations(app, &settings);
    for location in saved_locations(state) {
        if !list.iter().any(|existing| existing.path == location.path) {
            list.push(location);
        }
    }
    list
}

fn default_location_for(app: &AppHandle, state: &AppState) -> Option<Location> {
    let settings = state.settings_snapshot();
    let locations = all_locations(app, state);
    if settings.access_mode == AccessMode::Scoped {
        // Önce kullanıcının SAF ile eklediği konum.
        if let Some(scoped) = locations
            .iter()
            .find(|l| l.path.starts_with("content://"))
            .cloned()
        {
            return Some(scoped);
        }
    }
    locations
        .iter()
        .find(|l| l.id != "app-private")
        .or_else(|| locations.first())
        .cloned()
}

#[tauri::command]
pub fn list_locations(app: AppHandle, state: State<'_, AppState>) -> Vec<Location> {
    all_locations(&app, &state)
}

#[tauri::command]
pub fn default_location(app: AppHandle, state: State<'_, AppState>) -> Option<Location> {
    default_location_for(&app, &state)
}

#[tauri::command]
pub fn pick_location(app: AppHandle, state: State<'_, AppState>) -> AppResult<Location> {
    let picked: mobile::PickedDir = mobile::invoke(&app, "pickDirectory", json!({}))?;
    let location = Location {
        id: format!("loc-{}", util::now_secs()),
        label: picked.name,
        path: picked.uri,
        provider: "saf".into(),
        removable: false,
        added_at: util::now_secs(),
    };
    {
        let mut saved = state
            .locations
            .lock()
            .map_err(|_| AppError::Internal("konum kilidi".into()))?;
        saved.retain(|item| item.path != location.path);
        saved.push(location.clone());
        settings::save_locations(&app, &saved[..])?;
    }
    Ok(location)
}

#[tauri::command]
pub fn add_location(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    label: Option<String>,
) -> AppResult<Location> {
    let entry = provider::stat(&app, &path)?;
    if !entry.is_dir {
        return Err(AppError::Unsupported("Konum bir klasör olmalı".into()));
    }
    let location = Location {
        id: format!("loc-{}", util::now_secs()),
        label: label.unwrap_or_else(|| entry.name.clone()),
        provider: provider::provider_name(&path).into(),
        removable: false,
        added_at: util::now_secs(),
        path,
    };
    let mut saved = state
        .locations
        .lock()
        .map_err(|_| AppError::Internal("konum kilidi".into()))?;
    saved.retain(|item| item.path != location.path);
    saved.push(location.clone());
    settings::save_locations(&app, &saved[..])?;
    Ok(location)
}

#[tauri::command]
pub fn remove_location(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Vec<Location>> {
    let mut saved = state
        .locations
        .lock()
        .map_err(|_| AppError::Internal("konum kilidi".into()))?;
    if let Some(index) = saved.iter().position(|item| item.id == id) {
        let location = saved.remove(index);
        if util::is_content_uri(&location.path) {
            let _: serde_json::Value =
                mobile::invoke(&app, "releasePermission", json!({ "uri": location.path }))
                    .unwrap_or(serde_json::Value::Null);
        }
        settings::save_locations(&app, &saved[..])?;
    }
    drop(saved);
    Ok(all_locations(&app, &state))
}

#[tauri::command]
pub fn storage_summary(app: AppHandle, path: String) -> AppResult<StorageInfo> {
    if util::is_content_uri(&path) {
        return SafProvider::new(&app)?.storage_info(&path);
    }
    let usage: mobile::StorageUsage = mobile::invoke(&app, "storageInfo", json!({ "path": path }))
        .map_err(|_| AppError::Unsupported("depolama bilgisi".into()))?;
    Ok(StorageInfo {
        total: usage.total,
        free: usage.free,
        used: usage.total.saturating_sub(usage.free),
    })
}

// ---------------------------------------------------------------------------
// Dosya sistemi
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_dir(app: AppHandle, state: State<'_, AppState>, path: String) -> AppResult<DirListing> {
    let settings = state.settings_snapshot();
    let mut entries = provider::list(&app, &path)?;
    if !settings.show_hidden {
        entries.retain(|entry| !util::is_hidden(&entry.name));
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let locations = all_locations(&app, &state);
    let is_root = locations.iter().any(|l| l.path == path);
    Ok(DirListing {
        breadcrumb: build_breadcrumb(&app, &path, &locations),
        path,
        entries,
        is_root,
    })
}

fn build_breadcrumb(app: &AppHandle, path: &str, locations: &[Location]) -> Vec<Crumb> {
    if util::is_content_uri(path) {
        let name = provider::stat(app, path)
            .map(|e| e.name)
            .unwrap_or_else(|_| util::display_name(path));
        return vec![Crumb {
            name,
            path: path.to_string(),
        }];
    }
    let root = locations
        .iter()
        .filter(|l| path.starts_with(&l.path) && !l.path.is_empty())
        .max_by_key(|l| l.path.len());
    let mut crumbs = Vec::new();
    let mut base = "/".to_string();
    if let Some(root) = root {
        crumbs.push(Crumb {
            name: root.label.clone(),
            path: root.path.clone(),
        });
        base = root.path.clone();
    }
    let rest = path
        .strip_prefix(&base)
        .unwrap_or(path)
        .trim_start_matches('/');
    for segment in rest.split('/').filter(|s| !s.is_empty()) {
        base = format!("{}/{}", base.trim_end_matches('/'), segment);
        crumbs.push(Crumb {
            name: segment.to_string(),
            path: base.clone(),
        });
    }
    crumbs
}

#[tauri::command]
pub fn stat_entry(app: AppHandle, path: String) -> AppResult<FileEntry> {
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn read_file(app: AppHandle, path: String, limit: Option<u64>) -> AppResult<ReadFileResult> {
    let entry = provider::stat(&app, &path)?;
    if entry.is_dir {
        return Err(AppError::Unsupported("Klasör okunamaz".into()));
    }
    let is_text = matches!(entry.category, FileCategory::Text | FileCategory::Document)
        && !entry.mime.contains("pdf")
        && !entry.mime.contains("word")
        && !entry.mime.contains("sheet")
        && !entry.mime.contains("presentation");
    let cap = if is_text {
        limit.unwrap_or(MAX_TEXT_BYTES).min(MAX_TEXT_BYTES)
    } else {
        limit.unwrap_or(MAX_BINARY_BYTES).min(MAX_BINARY_BYTES)
    };
    let data = provider::open(&app, &path)?.read(&path, cap)?;
    let truncated = entry.size > data.len() as u64;
    if is_text {
        Ok(ReadFileResult {
            text: Some(String::from_utf8_lossy(&data).to_string()),
            base64: None,
            size: entry.size,
            mime: entry.mime,
            truncated,
        })
    } else {
        Ok(ReadFileResult {
            text: None,
            base64: Some(base64::engine::general_purpose::STANDARD.encode(&data)),
            size: entry.size,
            mime: entry.mime,
            truncated,
        })
    }
}

#[tauri::command]
pub fn write_text_file(app: AppHandle, path: String, text: String) -> AppResult<FileEntry> {
    provider::open(&app, &path)?.write(&path, text.as_bytes())?;
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn write_binary_file(
    app: AppHandle,
    path: String,
    base64_data: String,
) -> AppResult<FileEntry> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(base64_data.as_bytes())
        .map_err(|_| AppError::Internal("base64 çözülemedi".into()))?;
    provider::open(&app, &path)?.write(&path, &data)?;
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn create_dir(app: AppHandle, parent: String, name: String) -> AppResult<FileEntry> {
    let path = provider::create_dir(&app, &parent, &name)?;
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn create_file(app: AppHandle, parent: String, name: String) -> AppResult<FileEntry> {
    let path = provider::create_file(&app, &parent, &name)?;
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn rename_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    new_name: String,
) -> AppResult<FileEntry> {
    let new_path = provider::rename(&app, &path, &new_name)?;
    if util::is_content_uri(&new_path) || util::is_content_uri(&path) {
        // SAF yeniden adlandırma sonrası eski URI geçersiz olabilir.
        recent_add(app.clone(), state, new_path.clone(), None).ok();
    }
    provider::stat(&app, &new_path)
}

#[tauri::command]
pub async fn copy_entries(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    sources: Vec<String>,
    dest_dir: String,
) -> AppResult<Vec<FileEntry>> {
    let registry = Arc::clone(&state.jobs);
    run_job(app, registry, "copy", job_id, move |ctx| {
        transfer::transfer_many(&ctx, sources, dest_dir, TransferMode::Copy)
            .map(|items| items.into_iter().map(|item| item.entry).collect())
    })
    .await
}

#[tauri::command]
pub async fn move_entries(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    sources: Vec<String>,
    dest_dir: String,
) -> AppResult<Vec<FileEntry>> {
    let registry = Arc::clone(&state.jobs);
    run_job(app, registry, "move", job_id, move |ctx| {
        transfer::transfer_many(&ctx, sources, dest_dir, TransferMode::Move)
            .map(|items| items.into_iter().map(|item| item.entry).collect())
    })
    .await
}

#[tauri::command]
pub async fn delete_entries(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    items: Vec<DeleteRequest>,
    location_label: String,
) -> AppResult<Vec<TrashItem>> {
    let registry = Arc::clone(&state.jobs);
    run_job(app, registry, "delete", job_id, move |ctx| {
        let pairs = items
            .into_iter()
            .map(|item| (item.path, item.parent))
            .collect();
        trash::enqueue_delete(&ctx, pairs, &location_label)
    })
    .await
}

#[tauri::command]
pub fn delete_permanent(app: AppHandle, paths: Vec<String>) -> AppResult<usize> {
    let mut removed = 0;
    for path in paths {
        provider::remove(&app, &path)?;
        removed += 1;
    }
    Ok(removed)
}

// ---------------------------------------------------------------------------
// Çöp kutusu
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn trash_list(app: AppHandle) -> Vec<TrashItem> {
    trash::list(&app)
}

#[tauri::command]
pub fn trash_size_bytes(app: AppHandle) -> u64 {
    trash::size(&app)
}

#[tauri::command]
pub fn trash_restore(app: AppHandle, id: String) -> AppResult<FileEntry> {
    trash::restore(&app, &id)
}

#[tauri::command]
pub fn trash_purge(app: AppHandle, id: String) -> AppResult<()> {
    trash::purge(&app, &id)
}

#[tauri::command]
pub fn trash_empty(app: AppHandle) -> AppResult<usize> {
    trash::empty(&app)
}

// ---------------------------------------------------------------------------
// Favoriler / son kullanılanlar
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn favorites_list(state: State<'_, AppState>) -> Vec<SavedPath> {
    state
        .favorites
        .lock()
        .map(|list| list.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn favorite_toggle(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> AppResult<bool> {
    let entry = provider::stat(&app, &path)?;
    let mut favorites = state
        .favorites
        .lock()
        .map_err(|_| AppError::Internal("favori kilidi".into()))?;
    let existing = favorites.iter().position(|item| item.path == path);
    let added = existing.is_none();
    if let Some(index) = existing {
        favorites.remove(index);
    } else {
        favorites.push(SavedPath::from_entry(&entry, None));
    }
    settings::save_favorites(&app, &favorites[..])?;
    Ok(added)
}

#[tauri::command]
pub fn recents_list(state: State<'_, AppState>) -> Vec<SavedPath> {
    state
        .recents
        .lock()
        .map(|list| list.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn recent_add(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    location_id: Option<String>,
) -> AppResult<Vec<SavedPath>> {
    let entry = provider::stat(&app, &path)?;
    let mut recents = state
        .recents
        .lock()
        .map_err(|_| AppError::Internal("son kullanılan kilidi".into()))?;
    settings::push_recent(&mut recents, SavedPath::from_entry(&entry, location_id));
    settings::save_recents(&app, &recents[..])?;
    Ok(recents.clone())
}

// ---------------------------------------------------------------------------
// Arama (akışlı)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn search_start(
    app: AppHandle,
    state: State<'_, AppState>,
    search_id: String,
    root: String,
    query: String,
    categories: Option<Vec<FileCategory>>,
) -> AppResult<()> {
    search::start(
        &app,
        &state.searches,
        search_id,
        root,
        query,
        categories.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn search_cancel(state: State<'_, AppState>, search_id: String) {
    state.searches.cancel(&search_id);
}

#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, job_id: String) -> bool {
    state.jobs.cancel(&job_id)
}

// ---------------------------------------------------------------------------
// Küçük resim, önizleme ve arşiv
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn thumbnail(app: AppHandle, path: String, size: u32) -> AppResult<ThumbResult> {
    tauri::async_runtime::spawn_blocking(move || thumbnails::generate(&app, &path, size))
        .await
        .map_err(|e| AppError::Internal(format!("küçük resim: {e}")))?
}

#[tauri::command]
pub async fn preview_asset(app: AppHandle, path: String) -> AppResult<PreviewAsset> {
    tauri::async_runtime::spawn_blocking(move || {
        let entry = provider::stat(&app, &path)?;
        if entry.is_dir {
            return Err(AppError::Unsupported("Klasör önizlenemez".into()));
        }
        let cached = thumbnails::preview_cache_path(&app, &path)?;
        Ok(PreviewAsset {
            path: cached.to_string_lossy().to_string(),
            source: entry.path.clone(),
            mime: entry.mime.clone(),
            size: entry.size,
            kind: format!("{:?}", entry.category).to_lowercase(),
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("önizleme: {e}")))?
}

#[tauri::command]
pub async fn zip_create(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    sources: Vec<String>,
    dest_dir: String,
    name: String,
) -> AppResult<ZipResult> {
    let registry = Arc::clone(&state.jobs);
    run_job(app, registry, "zip", job_id, move |ctx| {
        archive::zip_paths(&ctx, sources, dest_dir, name)
    })
    .await
}

#[tauri::command]
pub async fn zip_extract(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    archive: String,
    dest_dir: String,
) -> AppResult<ZipResult> {
    let registry = Arc::clone(&state.jobs);
    run_job(app, registry, "unzip", job_id, move |ctx| {
        archive::unzip(&ctx, archive, dest_dir)
    })
    .await
}

// ---------------------------------------------------------------------------
// Android entegrasyonu
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn share_files(app: AppHandle, paths: Vec<String>) -> AppResult<()> {
    let _: serde_json::Value = mobile::invoke(&app, "shareFiles", json!({ "paths": paths }))?;
    Ok(())
}

#[tauri::command]
pub fn open_with(app: AppHandle, path: String) -> AppResult<()> {
    let mime = provider::stat(&app, &path)
        .map(|e| e.mime)
        .unwrap_or_else(|_| "application/octet-stream".into());
    let _: serde_json::Value =
        mobile::invoke(&app, "openWith", json!({ "path": path, "mime": mime }))?;
    Ok(())
}

#[tauri::command]
pub fn incoming_files(app: AppHandle, state: State<'_, AppState>) -> Vec<mobile::IncomingFile> {
    let _ = &state;
    mobile::invoke::<mobile::IncomingResponse>(&app, "takeIncoming", json!({}))
        .map(|response| response.files)
        .unwrap_or_default()
}

#[tauri::command]
pub fn import_incoming(
    app: AppHandle,
    state: State<'_, AppState>,
    uri: String,
    dest_dir: String,
    name: Option<String>,
) -> AppResult<FileEntry> {
    let _ = &state;
    let source = provider::stat(&app, &uri)?;
    let dest_name = name.unwrap_or_else(|| source.name.clone());
    let unique = provider::unique_name_in(&app, &dest_dir, &dest_name)?;
    let cancel = provider::CancelToken::new();
    let progress = provider::ProgressSink::silent();
    let path = transfer::copy_path(&app, &uri, &dest_dir, &unique, &cancel, &progress)?;
    provider::stat(&app, &path)
}

#[tauri::command]
pub fn minimize_app(app: AppHandle) -> AppResult<()> {
    let _: serde_json::Value = mobile::invoke(&app, "minimizeApp", json!({}))?;
    Ok(())
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    let package = app.package_info();
    AppInfo {
        version: package.version.to_string(),
        identifier: package.name.clone(),
        platform: std::env::consts::OS.to_string(),
        bridge_available: mobile::is_available(&app),
    }
}

/// Android 12+ duvar kağıdı vurgu rengi. Desteklenmiyorsa `None` döner.
/// Android sistem çubuklarının simge rengi (açık/koyu tema).
#[tauri::command]
pub async fn set_system_bars(app: AppHandle, dark: bool) -> AppResult<()> {
    mobile::invoke::<serde_json::Value>(&app, "systemBars", json!({ "dark": dark }))?;
    Ok(())
}

#[tauri::command]
pub fn dynamic_accent(app: AppHandle) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Accent {
        color: String,
        available: bool,
    }
    mobile::invoke::<Accent>(&app, "dynamicAccent", json!({}))
        .ok()
        .filter(|accent| accent.available && accent.color.starts_with('#'))
        .map(|accent| accent.color)
}

/// Açılışta uygulama klasöründe bir karşılama dosyası yoksa oluşturur.
pub fn ensure_private_scaffold(app: &AppHandle) -> AppResult<()> {
    let dir: PathBuf = settings::app_private_dir(app)?;
    let readme = dir.join("Dosya Dolabı.txt");
    if !readme.exists() {
        std::fs::write(
            &readme,
            "Dosya Dolabı\n\nBu klasör uygulamaya aittir; izin gerektirmeden her zaman erişilebilir.\n",
        )?;
    }
    let saved = dir.join("Kaydedilenler");
    if !saved.exists() {
        std::fs::create_dir_all(&saved)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silme_istegi_serde() {
        let json = r#"{"path":"/a/b.txt","parent":"/a"}"#;
        let request: DeleteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.parent, "/a");
    }
}
