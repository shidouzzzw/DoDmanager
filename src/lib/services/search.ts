// SPDX-License-Identifier: GPL-3.0-or-later
//! Akışlı arama istemcisi. Rust tarafı sonuçları parti parti gönderir; bu
//! servis her yeni aramada önceki oturumu iptal eder.

import { call, listen } from './backend';
import type { FileCategory, SearchDone, SearchHit } from '../types';

interface SearchBatch {
  searchId: string;
  hits: SearchHit[];
}

export interface SearchHandlers {
  onHits: (hits: SearchHit[]) => void;
  onDone?: (done: SearchDone) => void;
  onError?: (error: unknown) => void;
}

export class SearchSession {
  private searchId: string | null = null;
  private unlisten: (() => void)[] = [];

  constructor(private readonly handlers: SearchHandlers) {}

  get activeId(): string | null {
    return this.searchId;
  }

  async start(root: string, query: string, categories: FileCategory[] = []): Promise<void> {
    await this.stop();
    const searchId =
      typeof crypto !== 'undefined' && 'randomUUID' in crypto
        ? crypto.randomUUID()
        : `search-${Date.now()}`;
    this.searchId = searchId;

    this.unlisten.push(
      await listen<SearchBatch>('search:results', (batch) => {
        if (batch.searchId === this.searchId) this.handlers.onHits(batch.hits);
      }),
    );
    this.unlisten.push(
      await listen<SearchDone>('search:done', (done) => {
        if (done.searchId !== this.searchId) return;
        this.handlers.onDone?.(done);
      }),
    );

    try {
      await call<void>('search_start', { searchId, root, query, categories });
    } catch (error) {
      this.handlers.onError?.(error);
      await this.stop();
    }
  }

  async stop(): Promise<void> {
    if (this.searchId) {
      await call<void>('search_cancel', { searchId: this.searchId }).catch(() => {});
      this.searchId = null;
    }
    for (const off of this.unlisten) off();
    this.unlisten = [];
  }
}
