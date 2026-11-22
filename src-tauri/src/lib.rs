// SPDX-License-Identifier: GPL-3.0-or-later
//! Dosya Dolabı — Tauri 2 / Android.
//!
//! Katmanlar: arayüz → `src/lib/services` → Tauri komutları (`commands`) →
//! `StorageProvider` (yerel dosya sistemi ya da SAF). Arayüz hiçbir zaman
//! dosya sistemine doğrudan erişmez.

pub mod archive;
pub mod commands;
pub mod error;
pub mod jobs;
pub mod mobile;
pub mod models;
pub mod provider;
pub mod search;
pub mod settings;
pub mod thumbnails;
pub mod transfer;
pub mod trash;
pub mod util;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_haptics::init())
        .plugin(mobile::plugin())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = commands::AppState::load(&handle);
            if let Err(err) = commands::ensure_private_scaffold(&handle) {
                eprintln!("başlangıç hazırlığı başarısız: {err}");
            }
            app.manage(state);
            // Android'de paylaşım/`Birlikte aç` ile gelen dosyalar eklenti
            // tarafında biriktirilir; arayüz açılışta `incoming_files` çağırır.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::access_status,
            commands::set_access_mode,
            commands::request_all_files_access,
            commands::open_app_settings,
            commands::list_locations,
            commands::default_location,
            commands::pick_location,
            commands::add_location,
            commands::remove_location,
            commands::storage_summary,
            commands::list_dir,
            commands::stat_entry,
            commands::read_file,
            commands::write_text_file,
            commands::write_binary_file,
            commands::create_dir,
            commands::create_file,
            commands::rename_entry,
            commands::copy_entries,
            commands::move_entries,
            commands::delete_entries,
            commands::delete_permanent,
            commands::trash_list,
            commands::trash_size_bytes,
            commands::trash_restore,
            commands::trash_purge,
            commands::trash_empty,
            commands::favorites_list,
            commands::favorite_toggle,
            commands::recents_list,
            commands::recent_add,
            commands::search_start,
            commands::search_cancel,
            commands::cancel_job,
            commands::thumbnail,
            commands::preview_asset,
            commands::zip_create,
            commands::zip_extract,
            commands::share_files,
            commands::open_with,
            commands::incoming_files,
            commands::import_incoming,
            commands::minimize_app,
            commands::app_info,
            commands::dynamic_accent,
            commands::set_system_bars,
        ])
        .run(tauri::generate_context!())
        .expect("Dosya Dolabı başlatılamadı");
}
