// SPDX-License-Identifier: GPL-3.0-or-later
//! Uzun süren işler (kopyala/taşı/zip/aç) için iptal kaydı ve ilerleme event'leri.
//! İlerleme bildirimleri 100 ms'de bir yayınlanır; arayüz 60 fps kaydırmayı
//! korurken ilerleme çubuğunu akıcı gösterir.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::error::AppError;
use crate::models::{JobFinished, JobProgress};
use crate::provider::{CancelToken, ProgressSink};

const EMIT_INTERVAL: Duration = Duration::from_millis(100);

pub const EVENT_PROGRESS: &str = "job:progress";
pub const EVENT_FINISHED: &str = "job:finished";

#[derive(Default)]
pub struct JobRegistry {
    jobs: Mutex<HashMap<String, CancelToken>>,
}

impl JobRegistry {
    pub fn insert(&self, id: &str, token: CancelToken) {
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.insert(id.to_string(), token);
        }
    }

    pub fn cancel(&self, id: &str) -> bool {
        let token = self.jobs.lock().ok().and_then(|jobs| jobs.get(id).cloned());
        match token {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    pub fn remove(&self, id: &str) {
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.remove(id);
        }
    }

    pub fn active(&self) -> Vec<String> {
        self.jobs
            .lock()
            .map(|jobs| jobs.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Tek bir işin durumu. `Arc` olarak iş parçacığına taşınır.
pub struct JobCtx {
    app: AppHandle,
    registry: Arc<JobRegistry>,
    pub id: String,
    pub kind: String,
    pub cancel: CancelToken,
    done: AtomicU64,
    total: AtomicU64,
    last_emit: Mutex<Instant>,
    finished: Mutex<bool>,
}

impl JobCtx {
    pub fn start(app: &AppHandle, registry: &Arc<JobRegistry>, kind: &str, id: &str) -> Arc<Self> {
        let cancel = CancelToken::new();
        registry.insert(id, cancel.clone());
        Arc::new(Self {
            app: app.clone(),
            registry: Arc::clone(registry),
            id: id.to_string(),
            kind: kind.to_string(),
            cancel,
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            last_emit: Mutex::new(Instant::now() - EMIT_INTERVAL),
            finished: Mutex::new(false),
        })
    }

    pub fn sink(self: &Arc<Self>) -> ProgressSink {
        let ctx = Arc::clone(self);
        ProgressSink::new(move |bytes| ctx.add(bytes))
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::SeqCst);
        self.emit(false, "");
    }

    pub fn set_current(&self, name: &str) {
        self.emit(true, name);
    }

    pub fn add(&self, bytes: u64) {
        self.done.fetch_add(bytes, Ordering::SeqCst);
        self.emit(false, "");
    }

    pub fn guard(&self) -> crate::error::AppResult<()> {
        self.cancel.guard()
    }

    pub fn app_handle(&self) -> AppHandle {
        self.app.clone()
    }

    fn emit(&self, force: bool, current: &str) {
        let now = Instant::now();
        let should = force
            || now.duration_since(*self.last_emit.lock().unwrap_or_else(|e| e.into_inner()))
                >= EMIT_INTERVAL;
        if !should {
            return;
        }
        if let Ok(mut last) = self.last_emit.lock() {
            *last = now;
        }
        let _ = self.app.emit(
            EVENT_PROGRESS,
            JobProgress {
                job_id: self.id.clone(),
                kind: self.kind.clone(),
                done: self.done.load(Ordering::SeqCst),
                total: self.total.load(Ordering::SeqCst),
                current: current.to_string(),
            },
        );
    }

    pub fn finish_ok(self: &Arc<Self>) {
        self.finish(None);
    }

    pub fn finish_err(self: &Arc<Self>, err: &AppError) {
        self.finish(Some(err));
    }

    fn finish(self: &Arc<Self>, err: Option<&AppError>) {
        if let Ok(mut finished) = self.finished.lock() {
            if *finished {
                return;
            }
            *finished = true;
        }
        self.registry.remove(&self.id);
        let _ = self.app.emit(
            EVENT_FINISHED,
            JobFinished {
                job_id: self.id.clone(),
                kind: self.kind.clone(),
                ok: err.is_none(),
                message: err.map(|e| e.to_string()),
                code: err.map(|e| e.code().to_string()),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iptal_kaydi_temizlenir() {
        let registry = Arc::new(JobRegistry::default());
        let token = CancelToken::new();
        registry.insert("a", token.clone());
        assert!(registry.cancel("a"));
        assert!(token.is_cancelled());
        registry.remove("a");
        assert!(registry.active().is_empty());
        assert!(!registry.cancel("a"));
    }
}
