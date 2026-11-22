// SPDX-License-Identifier: GPL-3.0-or-later
//! Arka uç hata kodlarını kullanıcıya anlaşılır mesajlara ve çözüm eylemlerine
//! çevirir. Metinler i18n dosyalarından gelir; arayüz ham hata metni göstermez.

import { t } from '../i18n';

export type ErrorCode =
  | 'NOT_FOUND'
  | 'PERMISSION_DENIED'
  | 'READ_ONLY'
  | 'ALREADY_EXISTS'
  | 'INVALID_NAME'
  | 'STORAGE_FULL'
  | 'UNREADABLE'
  | 'CANCELLED'
  | 'ARCHIVE_ERROR'
  | 'UNSUPPORTED'
  | 'ANDROID_ONLY'
  | 'IO_ERROR'
  | 'INTERNAL'
  | 'NETWORK'
  | 'UNKNOWN';

export type ErrorAction = 'settings' | 'retry' | 'choose_location' | 'rename' | 'none';

export interface AppErrorShape {
  code: ErrorCode;
  message: string;
  action: ErrorAction;
  detail?: string;
}

export const ERROR_CODES: ErrorCode[] = [
  'NOT_FOUND',
  'PERMISSION_DENIED',
  'READ_ONLY',
  'ALREADY_EXISTS',
  'INVALID_NAME',
  'STORAGE_FULL',
  'UNREADABLE',
  'CANCELLED',
  'ARCHIVE_ERROR',
  'UNSUPPORTED',
  'ANDROID_ONLY',
  'IO_ERROR',
  'INTERNAL',
  'NETWORK',
  'UNKNOWN',
];

const ACTIONS: Record<ErrorCode, ErrorAction> = {
  NOT_FOUND: 'retry',
  PERMISSION_DENIED: 'settings',
  READ_ONLY: 'choose_location',
  ALREADY_EXISTS: 'rename',
  INVALID_NAME: 'rename',
  STORAGE_FULL: 'settings',
  UNREADABLE: 'none',
  CANCELLED: 'none',
  ARCHIVE_ERROR: 'none',
  UNSUPPORTED: 'none',
  ANDROID_ONLY: 'none',
  IO_ERROR: 'retry',
  INTERNAL: 'retry',
  NETWORK: 'retry',
  UNKNOWN: 'retry',
};

export class AppError extends Error {
  readonly code: ErrorCode;
  readonly action: ErrorAction;
  readonly detail?: string;

  constructor(shape: { code: ErrorCode; message?: string; action?: ErrorAction; detail?: string }) {
    super(shape.message ?? t(`errors.${shape.code}`));
    this.name = 'AppError';
    this.code = shape.code;
    this.action = shape.action ?? ACTIONS[shape.code] ?? 'none';
    this.detail = shape.detail;
  }

  /** Hata mesajını seçili dilde yeniden üretir. */
  localized(): string {
    return t(`errors.${this.code}`);
  }

  toShape(): AppErrorShape {
    return {
      code: this.code,
      message: this.localized(),
      action: this.action,
      detail: this.detail,
    };
  }
}

/** Arka uçtan gelen `{ code, message }` gövdesini ya da herhangi bir hatayı sarar. */
export function toAppError(raw: unknown): AppError {
  const code = extractCode(raw);
  const detail = extractMessage(raw);
  return new AppError({ code, detail });
}

export function extractCode(raw: unknown): ErrorCode {
  if (typeof raw === 'object' && raw !== null) {
    const candidate = (raw as { code?: unknown }).code;
    if (typeof candidate === 'string' && (ERROR_CODES as string[]).includes(candidate)) {
      return candidate as ErrorCode;
    }
  }
  if (typeof raw === 'string' && (ERROR_CODES as string[]).includes(raw)) {
    return raw as ErrorCode;
  }
  return 'UNKNOWN';
}

function extractMessage(raw: unknown): string | undefined {
  if (typeof raw === 'string') return raw;
  if (raw instanceof Error) return raw.message;
  if (typeof raw === 'object' && raw !== null) {
    const message = (raw as { message?: unknown }).message;
    if (typeof message === 'string') return message;
  }
  return undefined;
}

export function errorActionLabel(action: ErrorAction): string {
  switch (action) {
    case 'settings':
      return t('errors.action.settings');
    case 'retry':
      return t('errors.action.retry');
    case 'choose_location':
      return t('errors.action.chooseLocation');
    case 'rename':
      return t('errors.action.rename');
    default:
      return '';
  }
}

/** Hata kodu için kısa etiket (rozetlerde kullanılır). */
export function errorLabel(code: ErrorCode): string {
  const labels: Partial<Record<ErrorCode, string>> = {
    PERMISSION_DENIED: t('errors.PERMISSION_DENIED').split('.')[0],
    STORAGE_FULL: t('common.error'),
  };
  return labels[code] ?? t('common.error');
}
