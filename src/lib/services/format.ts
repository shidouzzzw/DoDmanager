// SPDX-License-Identifier: GPL-3.0-or-later
//! Boyut, tarih ve dosya adı biçimlendirme. Tüm çıktı Türkçe yerel ayarına uyar.

const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  if (bytes < 1024) return `${bytes} B`;
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = value >= 100 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits).replace('.', ',')} ${UNITS[unit]}`;
}

export function formatDate(unixSeconds: number, locale = 'tr-TR'): string {
  if (!unixSeconds) return '—';
  const date = new Date(unixSeconds * 1000);
  const now = new Date();
  const sameYear = date.getFullYear() === now.getFullYear();
  return new Intl.DateTimeFormat(locale, {
    day: '2-digit',
    month: sameYear ? 'short' : 'short',
    year: sameYear ? undefined : 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

export function formatRelative(unixSeconds: number, locale = 'tr-TR'): string {
  if (!unixSeconds) return '—';
  const diff = Math.max(0, Date.now() / 1000 - unixSeconds);
  const formatter = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
  if (diff < 60) return formatter.format(-Math.round(diff), 'second');
  if (diff < 3600) return formatter.format(-Math.round(diff / 60), 'minute');
  if (diff < 86400) return formatter.format(-Math.round(diff / 3600), 'hour');
  if (diff < 604800) return formatter.format(-Math.round(diff / 86400), 'day');
  return formatDate(unixSeconds, locale);
}

export function extensionOf(name: string): string {
  const index = name.lastIndexOf('.');
  if (index <= 0 || index === name.length - 1) return '';
  return name.slice(index + 1).toUpperCase();
}

export function isHiddenName(name: string): boolean {
  return name.startsWith('.') && name !== '.' && name !== '..';
}

/** Uzun adları ortadan kısaltır (breadcrumb ve tek satır etiketler). */
export function middleEllipsis(name: string, max = 28): string {
  if (name.length <= max) return name;
  const keep = Math.floor((max - 1) / 2);
  return `${name.slice(0, keep)}…${name.slice(-keep)}`;
}
