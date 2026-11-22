// SPDX-License-Identifier: GPL-3.0-or-later
//! Ayar, konum, favori ve son kullanılan kayıtlarının kalıcı deposu.
//! JSON dosyaları atomik olarak (geçici dosya + rename) yazılır.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, Location, SavedPath};
use crate::util;

const SETTINGS_FILE: &str = "settings.json";
const LOCATIONS_FILE: &str = "locations.json";
const FAVORITES_FILE: &str = "favorites.json";
const RECENTS_FILE: &str = "recents.json";
const RECENTS_LIMIT: usize = 30;

pub fn data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Internal(format!("ayar klasörü: {e}")))?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn app_private_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("uygulama klasörü: {e}")))?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let raw = serde_json::to_vec_pretty(value)?;
    fs::write(&tmp, raw)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    match data_dir(app) {
        Ok(dir) => read_json(&dir.join(SETTINGS_FILE)),
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    write_json(&data_dir(app)?.join(SETTINGS_FILE), settings)
}

pub fn load_locations(app: &AppHandle) -> Vec<Location> {
    match data_dir(app) {
        Ok(dir) => read_json(&dir.join(LOCATIONS_FILE)),
        Err(_) => Vec::new(),
    }
}

pub fn save_locations(app: &AppHandle, locations: &[Location]) -> AppResult<()> {
    write_json(&data_dir(app)?.join(LOCATIONS_FILE), &locations.to_vec())
}

pub fn load_favorites(app: &AppHandle) -> Vec<SavedPath> {
    match data_dir(app) {
        Ok(dir) => read_json(&dir.join(FAVORITES_FILE)),
        Err(_) => Vec::new(),
    }
}

pub fn save_favorites(app: &AppHandle, favorites: &[SavedPath]) -> AppResult<()> {
    write_json(&data_dir(app)?.join(FAVORITES_FILE), &favorites.to_vec())
}

pub fn load_recents(app: &AppHandle) -> Vec<SavedPath> {
    match data_dir(app) {
        Ok(dir) => read_json(&dir.join(RECENTS_FILE)),
        Err(_) => Vec::new(),
    }
}

pub fn save_recents(app: &AppHandle, recents: &[SavedPath]) -> AppResult<()> {
    let mut trimmed = recents.to_vec();
    trimmed.sort_by_key(|b| std::cmp::Reverse(b.touched_at));
    trimmed.truncate(RECENTS_LIMIT);
    write_json(&data_dir(app)?.join(RECENTS_FILE), &trimmed)
}

/// Aynı yol için eski kaydı düşürüp başa ekler.
pub fn push_recent(list: &mut Vec<SavedPath>, item: SavedPath) {
    list.retain(|existing| existing.path != item.path);
    list.insert(0, item);
    list.truncate(RECENTS_LIMIT);
}

/// Uygulama klasörü: her modda erişilebilen, uygulamaya ait özel alan.
pub fn app_private_location(app: &AppHandle) -> AppResult<Location> {
    let dir = app_private_dir(app)?;
    Ok(Location {
        id: "app-private".into(),
        label: "Uygulama klasörü".into(),
        path: dir.to_string_lossy().to_string(),
        provider: "local".into(),
        removable: false,
        added_at: util::now_secs(),
    })
}

/// Uygulama içi çöp kutusu klasörü.
pub fn trash_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app_private_dir(app)?.join("trash");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ThemeMode;

    #[test]
    fn ayar_varsayilanlari_ve_serde() {
        let settings = AppSettings::default();
        let raw = serde_json::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.theme, ThemeMode::System);
        assert_eq!(parsed.locale, "tr");
        // Eksik alanlar varsayılanla tamamlanır.
        let partial: AppSettings = serde_json::from_str("{\"theme\":\"dark\"}").unwrap();
        assert_eq!(partial.theme, ThemeMode::Dark);
        assert_eq!(partial.view, crate::models::ViewMode::Grid);
    }

    #[test]
    fn son_kullanilanlar_tekrarsiz_ve_sinirli() {
        let mut list = Vec::new();
        for i in 0..40 {
            push_recent(
                &mut list,
                SavedPath {
                    path: format!("/x/{i}"),
                    name: format!("{i}"),
                    is_dir: false,
                    mime: "text/plain".into(),
                    touched_at: i,
                    location_id: None,
                },
            );
        }
        assert_eq!(list.len(), RECENTS_LIMIT);
        let repeated = SavedPath {
            path: "/x/0".into(),
            name: "0".into(),
            is_dir: false,
            mime: "text/plain".into(),
            touched_at: 99,
            location_id: None,
        };
        push_recent(&mut list, repeated.clone());
        assert_eq!(list[0].path, "/x/0");
        assert_eq!(list.iter().filter(|p| p.path == "/x/0").count(), 1);
    }
}
