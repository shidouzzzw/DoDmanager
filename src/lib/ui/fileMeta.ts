// SPDX-License-Identifier: GPL-3.0-or-later
//! Dosya türüne göre ikon/renk seçimi, sıralama, filtreleme ve önizleme
//! kararları. Bileşenlerden bağımsız olduğu için birim testleriyle doğrulanır.

import { localeTag } from '../i18n';
import type { FileCategory, FileEntry, SortKey } from '../types';

export type FilterKey = 'all' | 'image' | 'video' | 'document' | 'audio' | 'archive';

const ICONS: Record<FileCategory, string> = {
  image: 'image',
  video: 'film',
  audio: 'music',
  document: 'file-text',
  archive: 'archive',
  text: 'file-code',
  folder: 'folder',
  other: 'file',
};

const FILTERS: Record<FilterKey, FileCategory[]> = {
  all: [],
  image: ['image'],
  video: ['video'],
  document: ['document', 'text'],
  audio: ['audio'],
  archive: ['archive'],
};

export function categoryIcon(entry: FileEntry): string {
  return entry.isDir ? 'folder' : (ICONS[entry.category] ?? 'file');
}

export function categoryColorVar(entry: FileEntry): string {
  return `var(--cat-${entry.isDir ? 'folder' : entry.category})`;
}

/** Dosya türü etiketi için i18n anahtarı. */
export function categoryLabelKey(entry: FileEntry): string {
  switch (entry.category) {
    case 'image':
      return 'browser.filterImages';
    case 'video':
      return 'browser.filterVideos';
    case 'audio':
      return 'browser.filterAudio';
    case 'archive':
      return 'browser.filterArchives';
    case 'folder':
      return 'common.folder';
    case 'document':
    case 'text':
      return 'browser.filterDocuments';
    default:
      return 'common.file';
  }
}

export function filterCategories(key: FilterKey): FileCategory[] {
  return FILTERS[key] ?? [];
}

export function matchesFilter(entry: FileEntry, key: FilterKey): boolean {
  if (key === 'all') return true;
  if (entry.isDir) return true;
  return filterCategories(key).includes(entry.category);
}

export function filterEntries(entries: FileEntry[], key: FilterKey): FileEntry[] {
  if (key === 'all') return entries;
  return entries.filter((entry) => matchesFilter(entry, key));
}

/** Klasörler her zaman önce gelir; ardından seçilen anahtara göre sıralanır. */
export function sortEntries(entries: FileEntry[], key: SortKey, desc: boolean): FileEntry[] {
  const collator = new Intl.Collator(localeTag(), { sensitivity: 'base', numeric: true });
  return [...entries].sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    let result: number;
    if (key === 'date') {
      result = a.modified - b.modified;
    } else if (key === 'size') {
      result = a.size - b.size;
    } else if (key === 'type' && a.category !== b.category) {
      result = a.category.localeCompare(b.category, localeTag());
    } else {
      result = collator.compare(a.name, b.name);
    }
    return desc ? -result : result;
  });
}

export function prepareEntries(
  entries: FileEntry[],
  options: { sort: SortKey; desc: boolean; filter: FilterKey },
): FileEntry[] {
  return sortEntries(filterEntries(entries, options.filter), options.sort, options.desc);
}

export type InlinePreview =
  'image' | 'video' | 'audio' | 'pdf' | 'markdown' | 'code' | 'text' | 'none';

/** Satır içi önizlemenin türü. */
export function inlinePreview(entry: FileEntry): InlinePreview {
  if (entry.isDir) return 'none';
  const lower = entry.name.toLowerCase();
  if (entry.category === 'image' || entry.mime.startsWith('image/')) return 'image';
  if (entry.category === 'video' || entry.mime.startsWith('video/')) return 'video';
  if (entry.category === 'audio' || entry.mime.startsWith('audio/')) return 'audio';
  if (entry.mime === 'application/pdf' || lower.endsWith('.pdf')) return 'pdf';
  if (lower.endsWith('.md') || lower.endsWith('.markdown')) return 'markdown';
  if (entry.category === 'text') return 'code';
  if (entry.mime.startsWith('text/')) return 'text';
  return 'none';
}

export function previewableFiles(entries: FileEntry[]): FileEntry[] {
  return entries.filter((entry) => !entry.isDir && inlinePreview(entry) !== 'none');
}

export function isZip(entry: FileEntry): boolean {
  return !entry.isDir && entry.name.toLowerCase().endsWith('.zip');
}

/** Sürükle-bırak hedefi olabilir mi? */
export function isDropTarget(entry: FileEntry): boolean {
  return entry.isDir && !entry.readOnly;
}

/** Seçimi aralığa genişletmek için iki indeks arasındaki tüm indeksler. */
export function rangeIndexes(from: number, to: number): number[] {
  if (from < 0 || to < 0) return [];
  const start = Math.min(from, to);
  const end = Math.max(from, to);
  return Array.from({ length: end - start + 1 }, (_, index) => start + index);
}

/** Dosya adından yeni klasör adı önerisi: "Yeni klasör", "Yeni klasör (2)"… */
export function suggestName(entries: FileEntry[], base: string): string {
  const taken = new Set(entries.map((entry) => entry.name.toLowerCase()));
  if (!taken.has(base.toLowerCase())) return base;
  for (let index = 2; index < 1000; index += 1) {
    const candidate = `${base} (${index})`;
    if (!taken.has(candidate.toLowerCase())) return candidate;
  }
  return `${base}-${Date.now()}`;
}
