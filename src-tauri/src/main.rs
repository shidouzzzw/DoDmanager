// SPDX-License-Identifier: GPL-3.0-or-later
// Android hedefi: giriş noktası `tauri::mobile_entry_point` ile lib içinde.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dod_manager_lib::run()
}
