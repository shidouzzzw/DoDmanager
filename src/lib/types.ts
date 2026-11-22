// SPDX-License-Identifier: GPL-3.0-or-later
/** Rust tarafındaki DTO'ların arayüz karşılıkları. Alanlar camelCase'tir. */

export type FileCategory =
  'image' | 'video' | 'audio' | 'document' | 'archive' | 'text' | 'folder' | 'other';

export interface FileEntry {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  mime: string;
  category: FileCategory;
  readOnly: boolean;
  childCount?: number | null;
}

export interface Crumb {
  name: string;
  path: string;
}

export interface DirListing {
  path: string;
  entries: FileEntry[];
  breadcrumb: Crumb[];
  isRoot: boolean;
}

export type AccessMode = 'scoped' | 'all_files';
export type ThemeMode = 'system' | 'light' | 'dark';
export type ViewMode = 'grid' | 'list';
export type SortKey = 'name' | 'date' | 'size' | 'type';

export interface AppSettings {
  locale: 'tr' | 'en';
  theme: ThemeMode;
  accent: string;
  dynamicColor: boolean;
  view: ViewMode;
  sort: SortKey;
  sortDesc: boolean;
  haptics: boolean;
  showHidden: boolean;
  reducedMotion: boolean;
  gridColumns: number;
  accessMode: AccessMode;
  onboardingComplete: boolean;
}

export interface Location {
  id: string;
  label: string;
  path: string;
  provider: 'local' | 'saf';
  removable: boolean;
  addedAt: number;
}

export interface AccessStatus {
  mode: AccessMode;
  hasAllFilesAccess: boolean;
  scopedLocations: number;
  defaultLocation: Location | null;
}

export interface StorageInfo {
  total: number;
  free: number;
  used: number;
}

export interface SavedPath {
  path: string;
  name: string;
  isDir: boolean;
  mime: string;
  touchedAt: number;
  locationId?: string | null;
}

export interface TrashItem {
  id: string;
  name: string;
  originalPath: string;
  originalParent: string;
  isDir: boolean;
  size: number;
  deletedAt: number;
  locationLabel: string;
}

export interface SearchHit {
  entry: FileEntry;
  parent: string;
}

export interface SearchDone {
  searchId: string;
  scanned: number;
  cancelled: boolean;
}

export interface JobProgress {
  jobId: string;
  kind: string;
  done: number;
  total: number;
  current: string;
}

export interface JobFinished {
  jobId: string;
  kind: string;
  ok: boolean;
  message: string | null;
  code: string | null;
}

export interface ReadFileResult {
  text: string | null;
  base64: string | null;
  size: number;
  mime: string;
  truncated: boolean;
}

export interface PreviewAsset {
  path: string;
  source: string;
  mime: string;
  size: number;
  kind: string;
}

export interface ThumbResult {
  path: string;
  cached: boolean;
  width: number;
  height: number;
}

export interface IncomingFile {
  uri: string;
  name: string;
  mime: string;
  size: number;
}

export interface AppInfo {
  version: string;
  identifier: string;
  platform: string;
  bridgeAvailable: boolean;
}

export interface ZipResult {
  output: string;
  entries: number;
  bytes: number;
}

export interface DeleteRequest {
  path: string;
  parent: string;
}

export type TransferMode = 'copy' | 'move';
