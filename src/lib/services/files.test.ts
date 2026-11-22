// SPDX-License-Identifier: GPL-3.0-or-later
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  convertFileSrc: (path: string) => `asset://localhost/${encodeURIComponent(path)}`,
}));

const originalWindow = globalThis.window;

beforeAll(() => {
  // `isTauri()` kontrolü için köprü işaretini taklit et.
  Object.defineProperty(globalThis, 'window', {
    value: { __TAURI_INTERNALS__: {} },
    configurable: true,
    writable: true,
  });
});

afterAll(() => {
  Object.defineProperty(globalThis, 'window', {
    value: originalWindow,
    configurable: true,
    writable: true,
  });
});

beforeEach(() => {
  invokeMock.mockReset();
});

describe('dosya servisi', () => {
  it('list_dir komutunu doğru argümanlarla çağırır', async () => {
    invokeMock.mockResolvedValue({ path: '/a', entries: [], breadcrumb: [], isRoot: false });
    const { listDir } = await import('./files');
    const result = await listDir('/a');
    expect(invokeMock).toHaveBeenCalledWith('list_dir', { path: '/a' });
    expect(result.path).toBe('/a');
  });

  it('arka uç hatasını AppError yapar', async () => {
    invokeMock.mockRejectedValue({ code: 'NOT_FOUND' });
    const { listDir } = await import('./files');
    await expect(listDir('/yok')).rejects.toMatchObject({ code: 'NOT_FOUND', action: 'retry' });
  });

  it('kopyalama iş kimliğini iletir', async () => {
    invokeMock.mockResolvedValue([]);
    const { copyEntries } = await import('./files');
    await copyEntries('job-1', ['/a/x'], '/b');
    expect(invokeMock).toHaveBeenCalledWith('copy_entries', {
      jobId: 'job-1',
      sources: ['/a/x'],
      destDir: '/b',
    });
  });

  it('silme isteklerini olduğu gibi gönderir', async () => {
    invokeMock.mockResolvedValue([]);
    const { deleteEntries } = await import('./files');
    await deleteEntries('job-2', [{ path: '/a/x', parent: '/a' }], 'Dahili depolama');
    expect(invokeMock).toHaveBeenCalledWith('delete_entries', {
      jobId: 'job-2',
      items: [{ path: '/a/x', parent: '/a' }],
      locationLabel: 'Dahili depolama',
    });
  });

  it('önbellek yolunu asset URL yapar', async () => {
    const { assetUrl } = await import('./files');
    expect(assetUrl('/cache/x.jpg')).toContain('asset://localhost/');
  });
});
