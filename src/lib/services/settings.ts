// SPDX-License-Identifier: GPL-3.0-or-later
//! Ayar, konum, erişim modeli ve uygulama geneli servisler.

import { call } from './backend';
import type {
  AccessMode,
  AccessStatus,
  AppInfo,
  AppSettings,
  IncomingFile,
  Location,
  SavedPath,
  StorageInfo,
  TrashItem,
} from '../types';

export function getSettings(): Promise<AppSettings> {
  return call<AppSettings>('get_settings');
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return call<AppSettings>('update_settings', { settings });
}

export function accessStatus(): Promise<AccessStatus> {
  return call<AccessStatus>('access_status');
}

export function setAccessMode(mode: AccessMode): Promise<AccessStatus> {
  return call<AccessStatus>('set_access_mode', { mode });
}

export function requestAllFilesAccess(): Promise<void> {
  return call<void>('request_all_files_access');
}

export function openAppSettings(): Promise<void> {
  return call<void>('open_app_settings');
}

export function listLocations(): Promise<Location[]> {
  return call<Location[]>('list_locations');
}

export function defaultLocation(): Promise<Location | null> {
  return call<Location | null>('default_location');
}

export function pickLocation(): Promise<Location> {
  return call<Location>('pick_location');
}

export function addLocation(path: string, label?: string): Promise<Location> {
  return call<Location>('add_location', { path, label });
}

export function removeLocation(id: string): Promise<Location[]> {
  return call<Location[]>('remove_location', { id });
}

export function storageSummary(path: string): Promise<StorageInfo> {
  return call<StorageInfo>('storage_summary', { path });
}

export function favoritesList(): Promise<SavedPath[]> {
  return call<SavedPath[]>('favorites_list');
}

export function favoriteToggle(path: string): Promise<boolean> {
  return call<boolean>('favorite_toggle', { path });
}

export function recentsList(): Promise<SavedPath[]> {
  return call<SavedPath[]>('recents_list');
}

export function recentAdd(path: string, locationId?: string): Promise<SavedPath[]> {
  return call<SavedPath[]>('recent_add', { path, locationId });
}

export function trashList(): Promise<TrashItem[]> {
  return call<TrashItem[]>('trash_list');
}

export function trashSizeBytes(): Promise<number> {
  return call<number>('trash_size_bytes');
}

export function trashRestore(id: string): Promise<unknown> {
  return call('trash_restore', { id });
}

export function trashPurge(id: string): Promise<void> {
  return call<void>('trash_purge', { id });
}

export function trashEmpty(): Promise<number> {
  return call<number>('trash_empty');
}

export function incomingFiles(): Promise<IncomingFile[]> {
  return call<IncomingFile[]>('incoming_files');
}

export function appInfo(): Promise<AppInfo> {
  return call<AppInfo>('app_info');
}

/** Android 12+ duvar kağıdı vurgu rengi (hex) ya da null. */
export function dynamicAccent(): Promise<string | null> {
  return call<string | null>('dynamic_accent');
}

/** Sistem çubuğu simgelerini temaya uydurur (kenardan kenara çizim). */
export function setSystemBars(dark: boolean): Promise<void> {
  return call<void>('set_system_bars', { dark });
}

/** Bağlantıyı sistem tarayıcısında açar (resmî opener eklentisi). */
export async function openExternal(url: string): Promise<void> {
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  await openUrl(url);
}

/** Yıkıcı işlemler için yerel onay penceresi (resmî dialog eklentisi). */
export async function askConfirm(message: string, title?: string): Promise<boolean> {
  try {
    const { ask } = await import('@tauri-apps/plugin-dialog');
    return await ask(message, {
      title: title ?? '',
      kind: 'warning',
      okLabel: 'Evet',
      cancelLabel: 'Vazgeç',
    });
  } catch {
    return false;
  }
}
