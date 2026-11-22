// SPDX-License-Identifier: GPL-3.0-or-later
//! Tauri `invoke` sarmalayıcısı. Tüm servisler buradan geçer; hatalar tipli
//! `AppError`'a dönüştürülür. Tarayıcıda (Tauri dışı) çalışırken anlaşılır bir
//! hata üretir, böylece arayüz geliştirme sırasında çökmez.

import { invoke } from '@tauri-apps/api/core';
import { AppError, toAppError } from './errors';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new AppError({
      code: 'ANDROID_ONLY',
      message: 'Dosya işlemleri için uygulamayı Tauri içinde çalıştır.',
      action: 'none',
    });
  }
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
}

/** Event dinleyici kaydı; Tauri dışında sessizce yok sayılır. */
export async function listen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (!isTauri()) return () => {};
  try {
    const { listen: tauriListen } = await import('@tauri-apps/api/event');
    const unlisten = await tauriListen<T>(event, (payload) => handler(payload.payload));
    return unlisten;
  } catch {
    return () => {};
  }
}
