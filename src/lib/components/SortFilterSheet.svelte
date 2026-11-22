<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { app } from '../state/app.svelte';
  import type { SortKey, ViewMode } from '../types';
  import type { FilterKey } from '../ui/fileMeta';
  import BottomSheet from './BottomSheet.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    filter: FilterKey;
    onfilter: (filter: FilterKey) => void;
    onclose: () => void;
  }

  let { filter, onfilter, onclose }: Props = $props();

  const sortOptions: { key: SortKey; label: string; icon: string }[] = [
    { key: 'name', label: t('browser.sortName'), icon: 'type' },
    { key: 'date', label: t('browser.sortDate'), icon: 'refresh-cw' },
    { key: 'size', label: t('browser.sortSize'), icon: 'hard-drive' },
    { key: 'type', label: t('browser.sortType'), icon: 'file-code' },
  ];

  const filters: { key: FilterKey; label: string }[] = [
    { key: 'all', label: t('browser.filterAll') },
    { key: 'image', label: t('browser.filterImages') },
    { key: 'video', label: t('browser.filterVideos') },
    { key: 'document', label: t('browser.filterDocuments') },
    { key: 'audio', label: t('browser.filterAudio') },
    { key: 'archive', label: t('browser.filterArchives') },
  ];

  function setView(view: ViewMode) {
    void app.updateSettings({ view });
  }
</script>

<BottomSheet title={t('browser.sort')} {onclose}>
  <div class="section">
    <p class="section-title">{t('browser.sort')}</p>
    <div class="sort-grid">
      {#each sortOptions as option (option.key)}
        <button
          class="sort"
          class:active={app.settings.sort === option.key}
          aria-pressed={app.settings.sort === option.key}
          onclick={() => app.updateSettings({ sort: option.key })}
        >
          <Icon name={option.icon} size={18} />
          {option.label}
        </button>
      {/each}
    </div>
    <button
      class="chip direction"
      aria-pressed={app.settings.sortDesc}
      onclick={() => app.updateSettings({ sortDesc: !app.settings.sortDesc })}
    >
      <Icon name={app.settings.sortDesc ? 'arrow-down' : 'arrow-up'} size={16} />
      {app.settings.sortDesc ? t('browser.desc') : t('browser.asc')}
    </button>
  </div>

  <div class="section">
    <p class="section-title">{t('browser.filter')}</p>
    <div class="chips">
      {#each filters as item (item.key)}
        <button
          class="chip"
          class:is-active={filter === item.key}
          aria-pressed={filter === item.key}
          onclick={() => onfilter(item.key)}
        >
          {item.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="section">
    <p class="section-title">{t('settings.appearance')}</p>
    <div class="chips">
      <button
        class="chip"
        class:is-active={app.settings.view === 'grid'}
        aria-pressed={app.settings.view === 'grid'}
        onclick={() => setView('grid')}
      >
        <Icon name="layout-grid" size={16} />
        {t('browser.viewGrid')}
      </button>
      <button
        class="chip"
        class:is-active={app.settings.view === 'list'}
        aria-pressed={app.settings.view === 'list'}
        onclick={() => setView('list')}
      >
        <Icon name="list" size={16} />
        {t('browser.viewList')}
      </button>
    </div>
    {#if app.settings.view === 'grid'}
      <div class="columns">
        <span>{t('settings.columns')}</span>
        <div class="stepper">
          <button
            class="icon-btn"
            aria-label={t('settings.columns')}
            onclick={() =>
              app.updateSettings({ gridColumns: Math.max(2, app.settings.gridColumns - 1) })}
          >
            <Icon name="chevron-left" size={18} />
          </button>
          <strong>{app.settings.gridColumns}</strong>
          <button
            class="icon-btn"
            aria-label={t('settings.columns')}
            onclick={() =>
              app.updateSettings({ gridColumns: Math.min(5, app.settings.gridColumns + 1) })}
          >
            <Icon name="chevron-right" size={18} />
          </button>
        </div>
      </div>
    {/if}
  </div>

  <div class="section">
    <label class="toggle-row">
      <Icon name="eye-off" size={18} />
      <span>{t('settings.showHidden')}</span>
      <input
        type="checkbox"
        checked={app.settings.showHidden}
        onchange={(event) =>
          app.updateSettings({ showHidden: (event.currentTarget as HTMLInputElement).checked })}
      />
    </label>
  </div>
</BottomSheet>

<style>
  .section {
    padding: var(--space-2) var(--space-5) var(--space-4);
    display: grid;
    gap: var(--space-3);
  }
  .section-title {
    font-family: var(--font-display);
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-3);
  }
  .sort-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
  }
  .sort {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 48px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--line);
    background: var(--paper);
    font-size: 0.875rem;
  }
  .sort.active {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .direction {
    justify-self: start;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .columns {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.875rem;
  }
  .stepper {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    padding: 2px;
  }
  .stepper strong {
    min-width: 20px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 52px;
    font-size: 0.9375rem;
  }
  .toggle-row span {
    flex: 1;
  }
  .toggle-row input {
    width: 24px;
    height: 24px;
    accent-color: var(--accent);
  }
</style>
