// SPDX-License-Identifier: GPL-3.0-or-later
import { beforeEach, describe, expect, it } from 'vitest';
import { extractCode, toAppError } from './errors';
import { setLocale } from '../i18n';

beforeEach(() => setLocale('tr'));

describe('hata eşleme', () => {
  it('izin hatasını ayarlara yönlendirir', () => {
    const error = toAppError({ code: 'PERMISSION_DENIED', message: 'ham' });
    expect(error.code).toBe('PERMISSION_DENIED');
    expect(error.action).toBe('settings');
    expect(error.localized()).toContain('izin');
  });

  it('ad çakışmasını yeniden adlandırmaya yönlendirir', () => {
    const error = toAppError({ code: 'ALREADY_EXISTS' });
    expect(error.action).toBe('rename');
    expect(error.localized()).toContain('Aynı adda');
  });

  it('depo dolmasında çöp kutusu önerir', () => {
    const error = toAppError({ code: 'STORAGE_FULL' });
    expect(error.localized()).toContain('doldu');
    expect(error.action).toBe('settings');
  });

  it('bilinmeyen kodu UNKNOWN yapar', () => {
    expect(extractCode({ code: 'BIR_SEY' })).toBe('UNKNOWN');
    expect(toAppError('tamamen metin').code).toBe('UNKNOWN');
    expect(toAppError(new Error('boom')).detail).toBe('boom');
  });

  it('iptal edilen işlem eylem istemez', () => {
    const error = toAppError({ code: 'CANCELLED' });
    expect(error.action).toBe('none');
    expect(error.localized()).toContain('iptal');
  });

  it('İngilizce mesaj üretir', () => {
    setLocale('en');
    expect(toAppError({ code: 'NOT_FOUND' }).localized()).toContain('no longer here');
  });
});
