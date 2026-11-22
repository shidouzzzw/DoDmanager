<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { formatSize } from '../services/format';
  import type { StorageInfo } from '../types';
  import Icon from './Icon.svelte';

  interface Props {
    storage: StorageInfo | null;
    loading?: boolean;
  }

  let { storage, loading = false }: Props = $props();
  const ratio = $derived(storage && storage.total > 0 ? storage.used / storage.total : 0);
</script>

<section class="card meter" aria-label={t('settings.storage')}>
  <header>
    <span class="label">{t('common.free')}</span>
    <span class="free">{storage ? formatSize(storage.free) : '—'}</span>
    <span class="capacity">
      {#if storage}
        {t('home.storageOf', {
          used: formatSize(storage.used),
          total: formatSize(storage.total),
        })}
      {:else if loading}
        {t('common.loading')}
      {:else}
        {t('home.storageUnavailable')}
      {/if}
    </span>
  </header>
  <div class="track" aria-hidden="true">
    <div class="fill" style="width: {Math.max(2, ratio * 100)}%"></div>
    <div class="ticks"></div>
  </div>
  <footer>
    <span><Icon name="hard-drive" size={15} /> {t('common.used')}</span>
    <span class="pct">{Math.round(ratio * 100)}%</span>
  </footer>
</section>

<style>
  .meter {
    margin: 0 var(--space-4) var(--space-5);
    padding: var(--space-4) var(--space-4) var(--space-3);
    display: grid;
    gap: var(--space-3);
    background: var(--paper-raised);
  }
  header {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: baseline;
    gap: var(--space-2);
  }
  .label {
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-3);
  }
  .free {
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 620;
    letter-spacing: -0.03em;
    text-align: right;
  }
  .capacity {
    grid-column: 1 / -1;
    font-size: 0.8125rem;
    color: var(--ink-3);
  }
  .track {
    position: relative;
    height: 10px;
    border-radius: 999px;
    background: var(--paper-sunken);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--accent),
      color-mix(in srgb, var(--accent) 55%, var(--ink))
    );
    transition: width var(--dur-slow) var(--ease-out);
  }
  .ticks {
    position: absolute;
    inset: 0;
    background-image: repeating-linear-gradient(
      90deg,
      transparent 0,
      transparent calc(10% - 1px),
      color-mix(in srgb, var(--paper) 70%, transparent) calc(10% - 1px),
      color-mix(in srgb, var(--paper) 70%, transparent) 10%
    );
    opacity: 0.35;
  }
  footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: var(--ink-3);
  }
  footer span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .pct {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--ink-2);
  }
</style>
