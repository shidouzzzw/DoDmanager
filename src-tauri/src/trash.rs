// SPDX-License-Identifier: GPL-3.0-or-later
//! Uygulama içi çöp kutusu. Silinen öğeler uygulamanın özel alanına taşınır;
//! geri yükleme özgün üst klasöre yapılır. Kalıcı silme yalnızca kullanıcı
//! çöp kutusunu boşalttığında gerçekleşir.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::jobs::JobCtx;
use crate::models::{FileEntry, TransferMode, TrashItem};
use crate::provider::{self, CancelToken, ProgressSink};
use crate::settings;
use crate::transfer;
use crate::util;

const INDEX_FILE: &str = "index.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TrashIndex {
    items: Vec<TrashItem>,
}

fn index_path(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(settings::trash_dir(app)?.join(INDEX_FILE))
}

fn read_index(app: &AppHandle) -> TrashIndex {
    index_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_index(app: &AppHandle, index: &TrashIndex) -> AppResult<()> {
    let path = index_path(app)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(index)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

pub fn list(app: &AppHandle) -> Vec<TrashItem> {
    let mut items = read_index(app).items;
    items.sort_by_key(|b| std::cmp::Reverse(b.deleted_at));
    items
}

/// Öğeyi çöp kutusuna taşır; kalıcı silme yapmaz.
pub fn move_to_trash(
    app: &AppHandle,
    ctx: Option<&Arc<JobCtx>>,
    path: &str,
    parent: &str,
    location_label: &str,
) -> AppResult<TrashItem> {
    let entry = provider::stat(app, path)?;
    let cancel = ctx.map(|c| c.cancel.clone()).unwrap_or_default();
    let progress = ctx.map(|c| c.sink()).unwrap_or_else(ProgressSink::silent);
    let id = uuid::Uuid::new_v4().to_string();
    let payload_dir = settings::trash_dir(app)?.join(&id);
    fs::create_dir_all(&payload_dir)?;

    // Kopyala + kaynağı kaldır: yarım kalan taşımada veri kaybı olmaz.
    let moved = transfer::copy_path(
        app,
        path,
        &payload_dir.to_string_lossy(),
        &entry.name,
        &cancel,
        &progress,
    )?;
    if let Err(err) = provider::remove(app, path) {
        let _ = provider::remove(app, &moved);
        return Err(err);
    }

    let item = TrashItem {
        id,
        name: entry.name.clone(),
        original_path: path.to_string(),
        original_parent: parent.to_string(),
        is_dir: entry.is_dir,
        size: entry.size,
        deleted_at: util::now_secs(),
        location_label: location_label.to_string(),
    };

    let mut index = read_index(app);
    index.items.push(item.clone());
    write_index(app, &index)?;
    Ok(item)
}

/// Çöp kutusundan geri yükler; ad çakışırsa yeni ad verir.
pub fn restore(app: &AppHandle, id: &str) -> AppResult<FileEntry> {
    let mut index = read_index(app);
    let position = index
        .items
        .iter()
        .position(|item| item.id == id)
        .ok_or_else(|| AppError::NotFound(format!("çöp öğesi {id}")))?;
    let item = index.items[position].clone();
    let payload_dir = settings::trash_dir(app)?.join(&item.id);
    let payload = payload_dir.join(&item.name);
    if !payload.exists() {
        return Err(AppError::NotFound(item.name));
    }

    if !provider::is_dir(app, &item.original_parent).unwrap_or(false) {
        return Err(AppError::NotFound(item.original_parent));
    }
    let dest_name = provider::unique_name_in(app, &item.original_parent, &item.name)?;
    let cancel = CancelToken::new();
    let progress = ProgressSink::silent();
    let restored = transfer::copy_path(
        app,
        &payload.to_string_lossy(),
        &item.original_parent,
        &dest_name,
        &cancel,
        &progress,
    )?;

    let _ = fs::remove_dir_all(&payload_dir);
    index.items.remove(position);
    write_index(app, &index)?;

    provider::stat(app, &restored)
}

/// Çöp kutusundaki tek öğeyi kalıcı siler.
pub fn purge(app: &AppHandle, id: &str) -> AppResult<()> {
    let mut index = read_index(app);
    let Some(position) = index.items.iter().position(|item| item.id == id) else {
        return Err(AppError::NotFound(format!("çöp öğesi {id}")));
    };
    let dir = settings::trash_dir(app)?.join(&index.items[position].id);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    index.items.remove(position);
    write_index(app, &index)
}

/// Çöp kutusunu boşaltır; silinen öğe sayısını döndürür.
pub fn empty(app: &AppHandle) -> AppResult<usize> {
    let index = read_index(app);
    let count = index.items.len();
    let dir = settings::trash_dir(app)?;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().map(|n| n == INDEX_FILE).unwrap_or(false) {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            let _ = fs::remove_file(&path);
        }
    }
    write_index(app, &TrashIndex::default())?;
    Ok(count)
}

/// Çöp kutusunun disk kullanımı (bayt).
pub fn size(app: &AppHandle) -> u64 {
    fn walk(path: &Path) -> u64 {
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };
        entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let meta = e.metadata();
                match meta {
                    Ok(m) if m.is_dir() => walk(&e.path()),
                    Ok(m) => m.len(),
                    Err(_) => 0,
                }
            })
            .sum()
    }
    settings::trash_dir(app).map(|d| walk(&d)).unwrap_or(0)
}

/// Aynı adı taşıyan öğeyi bulur (testler ve doğrulama için).
pub fn contains(app: &AppHandle, name: &str) -> bool {
    read_index(app).items.iter().any(|item| item.name == name)
}

/// Silme işini iş parçacığında başlatır.
pub fn enqueue_delete(
    ctx: &Arc<JobCtx>,
    paths: Vec<(String, String)>,
    location_label: &str,
) -> AppResult<Vec<TrashItem>> {
    ctx.set_total(paths.len() as u64);
    let mut deleted = Vec::new();
    for (path, parent) in paths {
        ctx.guard()?;
        ctx.set_current(&util::display_name(&path));
        let item = move_to_trash(&ctx.app_handle(), Some(ctx), &path, &parent, location_label)?;
        ctx.add(1);
        deleted.push(item);
    }
    ctx.finish_ok();
    Ok(deleted)
}

/// Testlerde ve temizleme işlerinde kullanılan doğrudan kopyalama/taşıma.
pub fn move_payload(source: &Path, target_dir: &Path) -> AppResult<PathBuf> {
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| AppError::InvalidName("kaynak adı".into()))?;
    let target = target_dir.join(&name);
    fs::rename(source, &target)?;
    Ok(target)
}

impl std::fmt::Display for TransferMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferMode::Copy => write!(f, "copy"),
            TransferMode::Move => write!(f, "move"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cop_indeksi_serde_eder() {
        let index = TrashIndex {
            items: vec![TrashItem {
                id: "x".into(),
                name: "a.txt".into(),
                original_path: "/a/a.txt".into(),
                original_parent: "/a".into(),
                is_dir: false,
                size: 3,
                deleted_at: 1,
                location_label: "Dahili depolama".into(),
            }],
        };
        let raw = serde_json::to_string(&index).unwrap();
        let parsed: TrashIndex = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].original_parent, "/a");
    }

    #[test]
    fn payload_tasima() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("a.txt");
        fs::write(&source, b"veri").unwrap();
        let target_dir = dir.path().join("hedef");
        fs::create_dir(&target_dir).unwrap();
        let moved = move_payload(&source, &target_dir).unwrap();
        assert!(moved.exists());
    }
}
