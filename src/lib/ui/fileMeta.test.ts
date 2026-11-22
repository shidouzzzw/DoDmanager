// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import type { FileEntry } from '../types';
import {
  filterEntries,
  inlinePreview,
  isZip,
  prepareEntries,
  previewableFiles,
  rangeIndexes,
  sortEntries,
  suggestName,
} from './fileMeta';

function entry(partial: Partial<FileEntry> & { name: string }): FileEntry {
  return {
    path: `/x/${partial.name}`,
    isDir: false,
    size: 100,
    modified: 1000,
    mime: 'text/plain',
    category: 'text',
    readOnly: false,
    ...partial,
  };
}

const files: FileEntry[] = [
  entry({ name: 'foto.jpg', category: 'image', mime: 'image/jpeg', size: 3000, modified: 500 }),
  entry({ name: 'Beta', isDir: true, category: 'folder', size: 0 }),
  entry({ name: 'alfa.txt', size: 100, modified: 900 }),
  entry({ name: 'video.mp4', category: 'video', mime: 'video/mp4', size: 9000 }),
  entry({ name: 'arsiv.zip', category: 'archive', size: 50 }),
  entry({ name: 'alfa klasör', isDir: true, category: 'folder', size: 0 }),
];

describe('sıralama', () => {
  it('klasörleri önce getirir ve doğal sıralama yapar', () => {
    const sorted = sortEntries(files, 'name', false);
    expect(sorted.slice(0, 2).every((item) => item.isDir)).toBe(true);
    expect(sorted[2].name).toBe('alfa.txt');
  });

  it('boyuta göre azalan sıralar', () => {
    const sorted = sortEntries(files, 'size', true);
    expect(sorted.find((item) => item.name === 'video.mp4')?.size).toBe(9000);
    expect(sorted.findIndex((item) => item.name === 'video.mp4')).toBeLessThan(
      sorted.findIndex((item) => item.name === 'foto.jpg'),
    );
  });

  it('tarihe göre artan sıralar', () => {
    const sorted = sortEntries(files, 'date', false);
    const dates = sorted.filter((item) => !item.isDir).map((item) => item.modified);
    expect([...dates].sort((a, b) => a - b)).toEqual(dates);
  });
});

describe('filtreleme', () => {
  it('görselleri süzer, klasörleri korur', () => {
    const filtered = filterEntries(files, 'image');
    expect(filtered.some((item) => item.name === 'foto.jpg')).toBe(true);
    expect(filtered.some((item) => item.name === 'video.mp4')).toBe(false);
    expect(filtered.filter((item) => item.isDir).length).toBe(2);
  });

  it('belge filtresi metni kapsar', () => {
    const filtered = prepareEntries(files, { sort: 'name', desc: false, filter: 'document' });
    expect(filtered.some((item) => item.name === 'alfa.txt')).toBe(true);
  });
});

describe('önizleme kararları', () => {
  it('türe göre satır içi önizleme seçer', () => {
    expect(inlinePreview(entry({ name: 'a.jpg', category: 'image' }))).toBe('image');
    expect(
      inlinePreview(entry({ name: 'a.pdf', category: 'document', mime: 'application/pdf' })),
    ).toBe('pdf');
    expect(inlinePreview(entry({ name: 'a.md', mime: 'text/markdown' }))).toBe('markdown');
    expect(inlinePreview(entry({ name: 'a.rs' }))).toBe('code');
    expect(
      inlinePreview(entry({ name: 'a.bin', category: 'other', mime: 'application/x-bin' })),
    ).toBe('none');
    expect(inlinePreview(entry({ name: 'klasor', isDir: true }))).toBe('none');
  });

  it('önizlenebilir dosyaları listeler', () => {
    expect(previewableFiles(files).length).toBe(4);
  });

  it('zip dosyasını tanır', () => {
    expect(isZip(entry({ name: 'a.ZIP', category: 'archive' }))).toBe(true);
    expect(isZip(entry({ name: 'a.rar', category: 'archive' }))).toBe(false);
  });
});

describe('yardımcılar', () => {
  it('aralık indekslerini üretir', () => {
    expect(rangeIndexes(2, 5)).toEqual([2, 3, 4, 5]);
    expect(rangeIndexes(-1, 5)).toEqual([]);
  });

  it('çakışmayan klasör adı önerir', () => {
    expect(suggestName(files, 'Yeni klasör')).toBe('Yeni klasör');
    expect(suggestName([entry({ name: 'Yeni klasör', isDir: true })], 'Yeni klasör')).toBe(
      'Yeni klasör (2)',
    );
  });
});
