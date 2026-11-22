// SPDX-License-Identifier: GPL-3.0-or-later
//! Topluluk çevirilerine hazır basit i18n. Metinler JSON dosyalarında tutulur;
//! yeni bir dil eklemek için `locales/` altına aynı anahtarlarla dosya koyup
//! aşağıdaki `dictionaries` haritasına eklemek yeterlidir.

import tr from './locales/tr.json';
import en from './locales/en.json';

export type Locale = 'tr' | 'en';
export type Dictionary = Record<string, string>;

export const SUPPORTED_LOCALES: { code: Locale; name: string }[] = [
  { code: 'tr', name: 'Türkçe' },
  { code: 'en', name: 'English' },
];

const dictionaries: Record<Locale, Dictionary> = {
  tr: tr as Dictionary,
  en: en as Dictionary,
};

let current: Locale = 'tr';

export function setLocale(locale: Locale | string): void {
  current = (locale === 'en' ? 'en' : 'tr') as Locale;
  if (typeof document !== 'undefined') {
    document.documentElement.lang = current;
  }
}

export function getLocale(): Locale {
  return current;
}

export function localeTag(): string {
  return current === 'tr' ? 'tr-TR' : 'en-US';
}

export function hasKey(key: string, locale: Locale = current): boolean {
  return key in dictionaries[locale];
}

/** `{ad}` yer tutucularını doldurur. */
export function t(key: string, params?: Record<string, string | number>): string {
  const raw = dictionaries[current][key] ?? dictionaries.tr[key] ?? key;
  if (!params) return raw;
  return raw.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/** Çeviri dosyalarının anahtar eşitliğini denetler (test ve CI için). */
export function dictionariesInSync(): boolean {
  const keys = Object.keys(dictionaries.tr).sort();
  return SUPPORTED_LOCALES.every(({ code }) => {
    const other = Object.keys(dictionaries[code]).sort();
    return keys.length === other.length && keys.every((key, index) => key === other[index]);
  });
}

export function dictionaryKeys(locale: Locale): string[] {
  return Object.keys(dictionaries[locale]);
}
