// SPDX-License-Identifier: GPL-3.0-or-later
//! Uzun süren işlerin ilerleme ve bitiş event'leri.

import { listen } from './backend';
import type { JobFinished, JobProgress } from '../types';

export async function onJobProgress(handler: (progress: JobProgress) => void): Promise<() => void> {
  return listen<JobProgress>('job:progress', handler);
}

export async function onJobFinished(handler: (finished: JobFinished) => void): Promise<() => void> {
  return listen<JobFinished>('job:finished', handler);
}
