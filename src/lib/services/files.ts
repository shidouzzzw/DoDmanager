// SPDX-License-Identifier: GPL-3.0-or-later
//! Dosya işlemleri servis katmanı. Arayüz yalnızca bu fonksiyonları çağırır;
//! hiçbir bileşen `invoke` ya da yol mantığı içermez.

import { convertFileSrc } from '@tauri-apps/api/core';
import { call, isTauri } from './backend';
import type {
  DeleteRequest,
  DirListing,
  FileEntry,
  FileCategory,
  PreviewAsset,
  ReadFileResult,
  ThumbResult,
  TrashItem,
  ZipResult,
} from '../types';

export function newJobId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) return crypto.randomUUID();
  return `job-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function listDir(path: string): Promise<DirListing> {
  return call<DirListing>('list_dir', { path });
}

export function statEntry(path: string): Promise<FileEntry> {
  return call<FileEntry>('stat_entry', { path });
}

export function readFile(path: string, limit?: number): Promise<ReadFileResult> {
  return call<ReadFileResult>('read_file', { path, limit });
}

export function writeTextFile(path: string, text: string): Promise<FileEntry> {
  return call<FileEntry>('write_text_file', { path, text });
}

export function createDir(parent: string, name: string): Promise<FileEntry> {
  return call<FileEntry>('create_dir', { parent, name });
}

export function createFile(parent: string, name: string): Promise<FileEntry> {
  return call<FileEntry>('create_file', { parent, name });
}

export function renameEntry(path: string, newName: string): Promise<FileEntry> {
  return call<FileEntry>('rename_entry', { path, newName });
}

export function copyEntries(
  jobId: string,
  sources: string[],
  destDir: string,
): Promise<FileEntry[]> {
  return call<FileEntry[]>('copy_entries', { jobId, sources, destDir });
}

export function moveEntries(
  jobId: string,
  sources: string[],
  destDir: string,
): Promise<FileEntry[]> {
  return call<FileEntry[]>('move_entries', { jobId, sources, destDir });
}

export function deleteEntries(
  jobId: string,
  items: DeleteRequest[],
  locationLabel: string,
): Promise<TrashItem[]> {
  return call<TrashItem[]>('delete_entries', { jobId, items, locationLabel });
}

export function deletePermanent(paths: string[]): Promise<number> {
  return call<number>('delete_permanent', { paths });
}

export function thumbnail(path: string, size = 256): Promise<ThumbResult> {
  return call<ThumbResult>('thumbnail', { path, size });
}

export function previewAsset(path: string): Promise<PreviewAsset> {
  return call<PreviewAsset>('preview_asset', { path });
}

export function zipCreate(
  jobId: string,
  sources: string[],
  destDir: string,
  name: string,
): Promise<ZipResult> {
  return call<ZipResult>('zip_create', { jobId, sources, destDir, name });
}

export function zipExtract(jobId: string, archive: string, destDir: string): Promise<ZipResult> {
  return call<ZipResult>('zip_extract', { jobId, archive, destDir });
}

export function shareFiles(paths: string[]): Promise<void> {
  return call<void>('share_files', { paths });
}

export function openWith(path: string): Promise<void> {
  return call<void>('open_with', { path });
}

export function importIncoming(uri: string, destDir: string, name?: string): Promise<FileEntry> {
  return call<FileEntry>('import_incoming', { uri, destDir, name });
}

export function cancelJob(jobId: string): Promise<boolean> {
  return call<boolean>('cancel_job', { jobId });
}

export function minimizeApp(): Promise<void> {
  return call<void>('minimize_app');
}

/** Önizleme/oynatma için yerel önbellek yolunu asset URL'sine çevirir. */
export function assetUrl(path: string): string {
  if (!path || !isTauri()) return '';
  try {
    return convertFileSrc(path);
  } catch {
    return '';
  }
}

const FILTER_CATEGORIES: Record<string, FileCategory[]> = {
  image: ['image'],
  video: ['video'],
  document: ['document', 'text'],
  audio: ['audio'],
  archive: ['archive'],
};

export function categoryFilter(key: string): FileCategory[] {
  return FILTER_CATEGORIES[key] ?? [];
}

export function isArchiveEntry(entry: FileEntry): boolean {
  return entry.category === 'archive' && entry.name.toLowerCase().endsWith('.zip');
}
