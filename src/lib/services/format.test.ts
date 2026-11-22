// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import {
  extensionOf,
  formatDate,
  formatRelative,
  formatSize,
  isHiddenName,
  middleEllipsis,
} from './format';

describe('formatSize', () => {
  it('bayt ve kilobaytı Türkçe ondalıkla yazar', () => {
    expect(formatSize(0)).toBe('0 B');
    expect(formatSize(512)).toBe('512 B');
    expect(formatSize(1536)).toBe('1,50 KB');
    expect(formatSize(10 * 1024 * 1024)).toBe('10,0 MB');
  });

  it('geçersiz değerlerde tire gösterir', () => {
    expect(formatSize(-1)).toBe('—');
    expect(formatSize(Number.NaN)).toBe('—');
  });
});

describe('formatDate', () => {
  it('tarih ve saati biçimlendirir', () => {
    const stamp = Math.floor(new Date('2026-01-15T10:30:00Z').getTime() / 1000);
    expect(formatDate(stamp)).toMatch(/15/);
    expect(formatDate(0)).toBe('—');
  });
});

describe('formatRelative', () => {
  it('yakın zamanları göreli yazar', () => {
    const now = Math.floor(Date.now() / 1000);
    expect(formatRelative(now - 5)).toMatch(/saniye önce|şimdi/);
    expect(formatRelative(now - 3600)).toMatch(/saat|sa/);
  });
});

describe('isim yardımcıları', () => {
  it('gizli dosyaları tanır', () => {
    expect(isHiddenName('.nomedia')).toBe(true);
    expect(isHiddenName('foto.jpg')).toBe(false);
    expect(isHiddenName('.')).toBe(false);
  });

  it('uzantıyı büyük harfle döndürür', () => {
    expect(extensionOf('rapor.pdf')).toBe('PDF');
    expect(extensionOf('arsiv.tar.gz')).toBe('GZ');
    expect(extensionOf('uzantisiz')).toBe('');
  });

  it('uzun adı ortadan kısaltır', () => {
    const name = 'cok-uzun-bir-dosya-adi-ornegi-2026.pdf';
    const shortened = middleEllipsis(name, 20);
    expect(shortened.length).toBeLessThanOrEqual(21);
    expect(shortened).toContain('…');
  });
});
