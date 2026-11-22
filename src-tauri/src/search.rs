// SPDX-License-Identifier: GPL-3.0-or-later
//! Ad araması. Ayrı bir iş parçacığında genişlik öncelikli gezinir, sonuçları
//! partiler halinde `search:results` olarak akıtır. Kullanıcı yazmayı
//! sürdürdükçe arayüz yeni bir arama kimliği başlatıp eskisini iptal eder.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::error::AppResult;
use crate::models::{FileCategory, SearchDone, SearchHit};
use crate::provider::{self, CancelToken};

pub const EVENT_RESULTS: &str = "search:results";
pub const EVENT_DONE: &str = "search:done";
const MAX_DEPTH: usize = 16;
const MAX_RESULTS: usize = 500;
const MAX_DIRS: usize = 50_000;
const BATCH_SIZE: usize = 24;
const BATCH_INTERVAL: Duration = Duration::from_millis(120);

#[derive(Default)]
pub struct SearchRegistry {
    searches: Mutex<HashMap<String, CancelToken>>,
}

impl SearchRegistry {
    pub fn insert(&self, id: &str, token: CancelToken) {
        if let Ok(mut map) = self.searches.lock() {
            map.insert(id.to_string(), token);
        }
    }

    pub fn cancel(&self, id: &str) -> bool {
        let token = self.searches.lock().ok().and_then(|m| m.get(id).cloned());
        if let Some(token) = token {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn remove(&self, id: &str) {
        if let Ok(mut map) = self.searches.lock() {
            map.remove(id);
        }
    }

    pub fn cancel_all(&self) {
        if let Ok(map) = self.searches.lock() {
            for token in map.values() {
                token.cancel();
            }
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SearchBatch {
    search_id: String,
    hits: Vec<SearchHit>,
}

/// Aramayı arka planda başlatır; kimliği döndürür.
pub fn start(
    app: &AppHandle,
    registry: &Arc<SearchRegistry>,
    search_id: String,
    root: String,
    query: String,
    categories: Vec<FileCategory>,
) -> AppResult<()> {
    provider::stat(app, &root)?;
    let cancel = CancelToken::new();
    registry.insert(&search_id, cancel.clone());
    let registry = Arc::clone(registry);
    let app = app.clone();
    let needle = query.trim().to_lowercase();

    std::thread::spawn(move || {
        let mut hits: Vec<SearchHit> = Vec::new();
        let mut scanned = 0u64;
        let mut dirs = 0usize;
        let mut last_emit = Instant::now();
        let mut stack: Vec<(String, usize)> = vec![(root.clone(), 0)];
        let cancelled = || cancel.is_cancelled();

        while let Some((dir, depth)) = stack.pop() {
            if cancelled() || hits.len() >= MAX_RESULTS || dirs >= MAX_DIRS {
                break;
            }
            dirs += 1;
            let Ok(children) = provider::list(&app, &dir) else {
                continue;
            };
            for child in children {
                if cancelled() {
                    break;
                }
                scanned += 1;
                if child.is_dir && depth < MAX_DEPTH {
                    stack.push((child.path.clone(), depth + 1));
                }
                if !needle.is_empty() && !child.name.to_lowercase().contains(&needle) {
                    continue;
                }
                if !categories.is_empty() && !categories.contains(&child.category) {
                    continue;
                }
                hits.push(SearchHit {
                    parent: dir.clone(),
                    entry: child,
                });
                if hits.len() >= MAX_RESULTS {
                    break;
                }
            }
            // Parti gönderimi: sonuçlar yazarken akmaya devam eder.
            if hits.len() >= BATCH_SIZE || last_emit.elapsed() >= BATCH_INTERVAL {
                emit_batch(&app, &search_id, &mut hits);
                last_emit = Instant::now();
            }
            if cancelled() {
                break;
            }
        }

        emit_batch(&app, &search_id, &mut hits);
        registry.remove(&search_id);
        let _ = app.emit(
            EVENT_DONE,
            SearchDone {
                search_id: search_id.clone(),
                scanned,
                cancelled: cancelled(),
            },
        );
    });
    Ok(())
}

fn emit_batch(app: &AppHandle, search_id: &str, hits: &mut Vec<SearchHit>) {
    if hits.is_empty() {
        return;
    }
    let batch = std::mem::take(hits);
    let _ = app.emit(
        EVENT_RESULTS,
        SearchBatch {
            search_id: search_id.to_string(),
            hits: batch,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arama_kaydi_iptal_edilir() {
        let registry = SearchRegistry::default();
        let token = CancelToken::new();
        registry.insert("s1", token.clone());
        assert!(registry.cancel("s1"));
        assert!(token.is_cancelled());
        registry.remove("s1");
        assert!(!registry.cancel("s1"));
    }
}
