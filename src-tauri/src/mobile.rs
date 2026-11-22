// SPDX-License-Identifier: GPL-3.0-or-later
//! Kotlin tarafındaki `AndroidFsPlugin` ile konuşan köprü.
//! Android dışında derlenebilir kalması için çağrılar `cfg` ile ayrılır.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub const PLUGIN_NAME: &str = "androidfs";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedEntry {
    pub uri: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedDir {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootInfo {
    pub path: String,
    pub label: String,
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantStatus {
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageUsage {
    pub total: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingFile {
    pub uri: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UriResult {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntriesResponse {
    pub entries: Vec<SharedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootsResponse {
    pub roots: Vec<RootInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingResponse {
    pub files: Vec<IncomingFile>,
}

/// Android'de eklenti başlatılırken tutulan tutamaç.
#[derive(Default)]
pub struct MobileBridge {
    #[cfg(target_os = "android")]
    handle: std::sync::Mutex<Option<tauri::plugin::PluginHandle<tauri::Wry>>>,
    /// `ACTION_VIEW` / `ACTION_SEND` ile gelen dosyalar (Android).
    incoming: std::sync::Mutex<Vec<IncomingFile>>,
}

impl MobileBridge {
    #[cfg(target_os = "android")]
    pub fn install(&self, handle: tauri::plugin::PluginHandle<tauri::Wry>) {
        if let Ok(mut slot) = self.handle.lock() {
            *slot = Some(handle);
        }
    }

    pub fn push_incoming(&self, files: Vec<IncomingFile>) {
        if let Ok(mut queue) = self.incoming.lock() {
            for file in files {
                if !queue.iter().any(|f| f.uri == file.uri) {
                    queue.push(file);
                }
            }
        }
    }

    pub fn take_incoming(&self) -> Vec<IncomingFile> {
        match self.incoming.lock() {
            Ok(mut queue) => std::mem::take(&mut *queue),
            Err(_) => Vec::new(),
        }
    }
}

/// Android eklentisi hazır mı? (Ayarlar ekranı için bilgi amaçlı.)
pub fn is_available(app: &AppHandle) -> bool {
    #[cfg(target_os = "android")]
    {
        app.try_state::<MobileBridge>()
            .and_then(|state| state.handle.lock().ok().and_then(|h| h.clone()))
            .is_some()
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        false
    }
}

/// Kotlin komutunu çağırır.
pub fn invoke<R: DeserializeOwned>(
    app: &AppHandle,
    method: &str,
    payload: serde_json::Value,
) -> AppResult<R> {
    #[cfg(target_os = "android")]
    {
        let state = app
            .try_state::<MobileBridge>()
            .ok_or(AppError::AndroidOnly)?;
        let handle = {
            let guard = state
                .handle
                .lock()
                .map_err(|_| AppError::Internal("köprü kilidi".into()))?;
            guard.clone().ok_or(AppError::AndroidOnly)?
        };
        let _ = handle.app();
        handle
            .run_mobile_plugin::<R>(method, payload)
            .map_err(|err| {
                let text = err.to_string();
                if text.to_lowercase().contains("permission") {
                    AppError::PermissionDenied
                } else {
                    AppError::Internal(format!("{method}: {text}"))
                }
            })
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, method, payload);
        Err(AppError::AndroidOnly)
    }
}

/// Android eklentisini Tauri'ye kaydeder; tutamacı duruma yazar.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(PLUGIN_NAME)
        .setup(|app, api| {
            let bridge = MobileBridge::default();
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("com.dod.manager", "AndroidFsPlugin")?;
                bridge.install(handle);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (app, api, &bridge);
            }
            app.manage(bridge);
            Ok(())
        })
        .build()
}
