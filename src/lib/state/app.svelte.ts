// SPDX-License-Identifier: GPL-3.0-or-later
//! Uygulamanın tek merkezî durumu. Ekran geçişleri, tema, seçim ve gezinme
//! burada tutulur; bileşenler servisleri doğrudan çağırmaz.

import { SvelteSet } from 'svelte/reactivity';
import { setLocale } from '../i18n';
import { toAppError, type AppError } from '../services/errors';
import { setHapticsEnabled } from '../services/haptics';
import { listDir } from '../services/files';
import {
  accessStatus,
  appInfo,
  defaultLocation,
  dynamicAccent,
  favoritesList,
  getSettings,
  incomingFiles,
  listLocations,
  pickLocation,
  recentsList,
  setSystemBars,
  storageSummary,
  trashList,
  trashSizeBytes,
  updateSettings as saveSettings,
} from '../services/settings';
import type {
  AccessStatus,
  AppInfo,
  AppSettings,
  Crumb,
  DirListing,
  FileEntry,
  IncomingFile,
  JobProgress,
  Location,
  SavedPath,
  SearchHit,
  StorageInfo,
  TrashItem,
  TransferMode,
} from '../types';

export type Screen =
  'home' | 'browser' | 'settings' | 'trash' | 'preview' | 'onboarding' | 'picker';

export const DEFAULT_SETTINGS: AppSettings = {
  locale: 'tr',
  theme: 'system',
  accent: 'indigo',
  dynamicColor: false,
  view: 'grid',
  sort: 'name',
  sortDesc: false,
  haptics: true,
  showHidden: false,
  reducedMotion: false,
  gridColumns: 3,
  accessMode: 'scoped',
  onboardingComplete: false,
};

export interface PickerIntent {
  mode: TransferMode;
  paths: string[];
  returnScreen: Screen;
}

interface SnackbarState {
  message: string;
  undo?: (() => Promise<void>) | null;
}

class AppState {
  settings = $state<AppSettings>(DEFAULT_SETTINGS);
  ready = $state(false);
  screen = $state<Screen>('home');
  returnScreen = $state<Screen>('home');

  locations = $state<Location[]>([]);
  access = $state<AccessStatus | null>(null);
  storage = $state<StorageInfo | null>(null);
  favorites = $state<SavedPath[]>([]);
  recents = $state<SavedPath[]>([]);
  trash = $state<TrashItem[]>([]);
  trashBytes = $state(0);

  listing = $state<DirListing | null>(null);
  loading = $state(false);
  loadingError = $state<AppError | null>(null);
  navStack = $state<Crumb[]>([]);
  currentLocation = $state<Location | null>(null);
  filter = $state('all');
  query = $state('');
  searchHits = $state<SearchHit[]>([]);
  searching = $state(false);
  searchScanned = $state(0);
  selected = $state<Set<string>>(new Set());
  anchorIndex = $state(-1);

  previewItems = $state<FileEntry[]>([]);
  previewIndex = $state(0);
  previewFolder = $state('');

  picker = $state<PickerIntent | null>(null);
  sheet = $state<string | null>(null);
  snackbar = $state<SnackbarState | null>(null);
  error = $state<AppError | null>(null);
  job = $state<JobProgress | null>(null);
  jobKind = $state('');
  incoming = $state<IncomingFile[]>([]);
  info = $state<AppInfo | null>(null);
  dynamicColorValue = $state<string | null>(null);
  systemDark = $state(false);

  private snackbarTimer: ReturnType<typeof setTimeout> | null = null;

  // -------------------------------------------------------------------
  // Başlangıç
  // -------------------------------------------------------------------

  async init(): Promise<void> {
    const settings = await getSettings().catch(() => DEFAULT_SETTINGS);
    this.settings = settings;
    this.applySettings(settings);
    this.info = await appInfo().catch(() => null);
    this.access = await accessStatus().catch(() => null);
    if (settings.accessMode !== 'scoped' && this.access && !this.access.hasAllFilesAccess) {
      // İzin geri alınmışsa güvenli moda dön.
      this.settings = { ...this.settings, accessMode: 'scoped' };
    }
    await this.refreshLocations();
    await Promise.all([this.refreshFavorites(), this.refreshRecents()]);
    this.ready = true;
    if (!settings.onboardingComplete) {
      this.screen = 'onboarding';
    } else {
      this.screen = 'home';
      void this.refreshHome();
      void this.checkIncoming();
    }
  }

  applySettings(settings: AppSettings): void {
    setLocale(settings.locale);
    setHapticsEnabled(settings.haptics);
    void this.applyAppearance();
  }

  async applyAppearance(): Promise<void> {
    if (typeof document === 'undefined') return;
    const root = document.documentElement;
    const systemDark =
      typeof window !== 'undefined' && window.matchMedia
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
        : false;
    this.systemDark = systemDark;
    const resolved =
      this.settings.theme === 'system' ? (systemDark ? 'dark' : 'light') : this.settings.theme;
    root.dataset.theme = resolved;
    root.dataset.accent = this.settings.accent;
    root.dataset.reducedMotion = String(this.settings.reducedMotion);
    root.style.colorScheme = resolved;
    void setSystemBars(resolved === 'dark').catch(() => {});

    if (this.settings.dynamicColor) {
      const color = this.dynamicColorValue ?? (await dynamicAccent().catch(() => null));
      if (color) {
        this.dynamicColorValue = color;
        root.style.setProperty('--accent', color);
        root.style.setProperty('--accent-soft', hexToRgba(color, 0.16));
        root.style.setProperty('--on-accent', readableInk(color));
        return;
      }
    }
    root.style.removeProperty('--accent');
    root.style.removeProperty('--accent-soft');
    root.style.removeProperty('--on-accent');
  }

  async updateSettings(patch: Partial<AppSettings>): Promise<void> {
    const next = { ...this.settings, ...patch };
    this.settings = next;
    this.applySettings(next);
    try {
      const saved = await saveSettings(next);
      this.settings = saved;
    } catch (error) {
      this.fail(error);
    }
  }

  // -------------------------------------------------------------------
  // Konumlar ve ana ekran
  // -------------------------------------------------------------------

  async refreshLocations(): Promise<void> {
    this.locations = await listLocations().catch(() => []);
    this.access = await accessStatus().catch(() => this.access);
  }

  async refreshFavorites(): Promise<void> {
    this.favorites = await favoritesList().catch(() => []);
  }

  async refreshRecents(): Promise<void> {
    this.recents = await recentsList().catch(() => []);
  }

  async refreshTrash(): Promise<void> {
    this.trash = await trashList().catch(() => []);
    this.trashBytes = await trashSizeBytes().catch(() => 0);
  }

  async refreshHome(): Promise<void> {
    await this.refreshLocations();
    const primary = this.access?.defaultLocation ?? (await defaultLocation().catch(() => null));
    if (primary) {
      this.storage = await storageSummary(primary.path).catch(() => null);
    }
    await Promise.all([this.refreshFavorites(), this.refreshRecents(), this.refreshTrash()]);
  }

  async pickNewLocation(): Promise<Location | null> {
    try {
      const location = await pickLocation();
      await this.refreshLocations();
      await this.refreshHome();
      return location;
    } catch (error) {
      this.fail(error);
      return null;
    }
  }

  // -------------------------------------------------------------------
  // Gezinme
  // -------------------------------------------------------------------

  async openLocation(location: Location): Promise<void> {
    this.currentLocation = location;
    this.navStack = [{ name: location.label, path: location.path }];
    this.screen = 'browser';
    this.query = '';
    this.searchHits = [];
    this.clearSelection();
    await this.loadPath(location.path, true);
  }

  async openFolder(entry: FileEntry): Promise<void> {
    this.navStack = [...this.navStack, { name: entry.name, path: entry.path }];
    this.query = '';
    this.searchHits = [];
    this.clearSelection();
    await this.loadPath(entry.path);
  }

  async loadPath(path: string, showSkeleton = false): Promise<void> {
    if (showSkeleton) {
      this.listing = null;
      this.loading = true;
    }
    this.loadingError = null;
    try {
      const listing = await listDir(path);
      this.listing = listing;
    } catch (error) {
      this.loadingError = toAppError(error);
      this.listing = null;
    } finally {
      this.loading = false;
    }
  }

  async reload(): Promise<void> {
    const path = this.currentPath;
    if (path) await this.loadPath(path);
  }

  get currentPath(): string {
    return this.navStack.length ? this.navStack[this.navStack.length - 1].path : '';
  }

  get currentName(): string {
    return this.navStack.length ? this.navStack[this.navStack.length - 1].name : '';
  }

  async goHome(): Promise<void> {
    this.screen = 'home';
    this.clearSelection();
    this.query = '';
    this.searchHits = [];
    await this.refreshHome();
  }

  async goToSettings(): Promise<void> {
    this.returnScreen = this.screen === 'onboarding' ? 'home' : this.screen;
    this.screen = 'settings';
  }

  async goToTrash(): Promise<void> {
    this.returnScreen = 'home';
    this.screen = 'trash';
    await this.refreshTrash();
  }

  openPreview(entries: FileEntry[], index: number): void {
    this.previewItems = entries;
    this.previewIndex = index;
    this.previewFolder = this.currentPath;
    this.returnScreen = 'browser';
    this.screen = 'preview';
  }

  closePreview(): void {
    this.screen = this.returnScreen;
    this.previewItems = [];
  }

  /** Android geri tuşu: işlendiyse `true` (varsayılan davranış engellenir). */
  back(): boolean {
    if (this.sheet) {
      this.sheet = null;
      return true;
    }
    if (this.screen === 'preview') {
      this.closePreview();
      return true;
    }
    if (this.screen === 'picker') {
      this.picker = null;
      this.screen = 'browser';
      return true;
    }
    if (this.screen === 'settings' || this.screen === 'trash') {
      this.screen = this.returnScreen;
      return true;
    }
    if (this.screen === 'onboarding') {
      return true;
    }
    if (this.screen === 'browser') {
      if (this.selected.size > 0) {
        this.clearSelection();
        return true;
      }
      if (this.query) {
        this.query = '';
        this.searchHits = [];
        return true;
      }
      if (this.navStack.length > 1) {
        this.navStack = this.navStack.slice(0, -1);
        void this.loadPath(this.currentPath);
        return true;
      }
      void this.goHome();
      return true;
    }
    return false;
  }

  // -------------------------------------------------------------------
  // Seçim
  // -------------------------------------------------------------------

  get selection(): FileEntry[] {
    const entries = this.listing?.entries ?? [];
    return entries.filter((entry) => this.selected.has(entry.path));
  }

  isSelected(path: string): boolean {
    return this.selected.has(path);
  }

  toggleSelection(path: string): void {
    const next = new SvelteSet(this.selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    this.selected = next;
  }

  setSelection(paths: string[]): void {
    this.selected = new Set(paths);
  }

  selectRange(indexes: number[], entries: FileEntry[]): void {
    const next = new SvelteSet(this.selected);
    for (const index of indexes) {
      const entry = entries[index];
      if (entry) next.add(entry.path);
    }
    this.selected = next;
  }

  selectAll(entries: FileEntry[]): void {
    this.selected = new Set(entries.map((entry) => entry.path));
  }

  clearSelection(): void {
    this.selected = new Set();
    this.anchorIndex = -1;
  }

  // -------------------------------------------------------------------
  // Transfer hedefi
  // -------------------------------------------------------------------

  beginTransfer(mode: TransferMode, paths: string[]): void {
    this.picker = { mode, paths, returnScreen: 'browser' };
    this.screen = 'picker';
  }

  // -------------------------------------------------------------------
  // Bildirimler
  // -------------------------------------------------------------------

  toast(message: string, undo?: (() => Promise<void>) | null, timeout = 4200): void {
    if (this.snackbarTimer) clearTimeout(this.snackbarTimer);
    this.snackbar = { message, undo };
    this.snackbarTimer = setTimeout(() => {
      this.snackbar = null;
    }, timeout);
  }

  dismissSnackbar(): void {
    if (this.snackbarTimer) clearTimeout(this.snackbarTimer);
    this.snackbar = null;
  }

  fail(error: unknown): AppError {
    const appError = toAppError(error);
    this.error = appError;
    return appError;
  }

  dismissError(): void {
    this.error = null;
  }

  async checkIncoming(): Promise<void> {
    const files = await incomingFiles().catch(() => []);
    if (files.length) this.incoming = files;
  }

  dismissIncoming(): void {
    this.incoming = [];
  }
}

/** Hex rengi saydamlıkla birlikte rgba'ya çevirir. */
export function hexToRgba(hex: string, alpha: number): string {
  const clean = hex.replace('#', '');
  const full =
    clean.length === 3
      ? clean
          .split('')
          .map((c) => c + c)
          .join('')
      : clean.padEnd(6, '0').slice(0, 6);
  const value = Number.parseInt(full, 16);
  const red = (value >> 16) & 255;
  const green = (value >> 8) & 255;
  const blue = value & 255;
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

/** Arka plan rengine göre okunaklı metin rengi seçer. */
export function readableInk(hex: string): string {
  const clean = hex.replace('#', '');
  if (clean.length < 6) return '#fbf8f0';
  const value = Number.parseInt(clean.slice(0, 6), 16);
  const red = (value >> 16) & 255;
  const green = (value >> 8) & 255;
  const blue = value & 255;
  const luminance = (0.299 * red + 0.587 * green + 0.114 * blue) / 255;
  return luminance > 0.6 ? '#14161a' : '#fbf8f0';
}

export const app = new AppState();
