// SPDX-License-Identifier: GPL-3.0-or-later
import { afterEach, describe, expect, it } from 'vitest';
import { dictionariesInSync, dictionaryKeys, getLocale, hasKey, setLocale, t } from './index';

afterEach(() => setLocale('tr'));

describe('i18n', () => {
  it('tüm diller aynı anahtar kümesine sahip', () => {
    expect(dictionariesInSync()).toBe(true);
    expect(dictionaryKeys('tr').length).toBeGreaterThan(100);
  });

  it('yer tutucuları doldurur', () => {
    setLocale('tr');
    expect(t('home.storageFree', { size: '12 GB' })).toBe('12 GB boş alan');
    expect(t('browser.selected', { count: 3 })).toBe('3 seçili');
  });

  it('eksik anahtarda anahtarın kendisini döndürür', () => {
    expect(hasKey('app.name')).toBe(true);
    expect(t('olmayan.anahtar')).toBe('olmayan.anahtar');
  });

  it('dili değiştirir ve bilinmeyen dili Türkçeye düşürür', () => {
    setLocale('en');
    expect(getLocale()).toBe('en');
    setLocale('de');
    expect(getLocale()).toBe('tr');
  });
});
