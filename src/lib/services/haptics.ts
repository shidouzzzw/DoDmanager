// SPDX-License-Identifier: GPL-3.0-or-later
//! Dokunsal geri bildirim. Resmî `@tauri-apps/plugin-haptics` eklentisi
//! kullanılır; ayarlarda kapalıysa ya da masaüstünde hiçbir şey yapmaz.

import { isTauri } from './backend';

export type HapticKind = 'select' | 'impact' | 'drag' | 'success' | 'warning' | 'delete';

let enabled = true;

export function setHapticsEnabled(value: boolean): void {
  enabled = value;
}

export async function haptic(kind: HapticKind): Promise<void> {
  if (!enabled || !isTauri()) return;
  try {
    const plugin = await import('@tauri-apps/plugin-haptics');
    switch (kind) {
      case 'select':
        await plugin.selectionFeedback();
        break;
      case 'impact':
        await plugin.impactFeedback('light');
        break;
      case 'drag':
        await plugin.impactFeedback('medium');
        break;
      case 'success':
        await plugin.notificationFeedback('success');
        break;
      case 'warning':
        await plugin.notificationFeedback('warning');
        break;
      case 'delete':
        await plugin.notificationFeedback('error');
        break;
    }
  } catch {
    // Titreşim desteklenmiyorsa sessizce devam et.
  }
}
