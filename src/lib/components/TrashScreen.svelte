<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { formatRelative, formatSize } from '../services/format';
  import { askConfirm, trashEmpty, trashPurge, trashRestore } from '../services/settings';
  import { app } from '../state/app.svelte';
  import type { TrashItem } from '../types';
  import ActionSheet from './ActionSheet.svelte';
  import EmptyState from './EmptyState.svelte';
  import Icon from './Icon.svelte';

  let selected = $state<TrashItem | null>(null);
  let busy = $state(false);

  async function restore(item: TrashItem) {
    busy = true;
    try {
      await trashRestore(item.id);
      selected = null;
      await app.refreshTrash();
      app.toast(t('trash.restored'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function purge(item: TrashItem) {
    busy = true;
    try {
      await trashPurge(item.id);
      selected = null;
      await app.refreshTrash();
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function emptyTrash() {
    const confirmed = await askConfirm(t('trash.emptyConfirm'), t('trash.empty'));
    if (!confirmed) return;
    busy = true;
    try {
      await trashEmpty();
      await app.refreshTrash();
      app.toast(t('trash.emptied'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }
</script>

<div class="screen">
  <header class="appbar pad-x">
    <button class="icon-btn" aria-label={t('a11y.back')} onclick={() => app.back()}>
      <Icon name="chevron-left" size={24} />
    </button>
    <div class="titles">
      <h1>{t('trash.title')}</h1>
      <span>{t('trash.count', { count: app.trash.length })} · {formatSize(app.trashBytes)}</span>
    </div>
    {#if app.trash.length}
      <button class="btn btn-danger small" onclick={emptyTrash} disabled={busy}>
        {t('trash.empty')}
      </button>
    {/if}
  </header>

  <div class="scroll">
    {#if app.trash.length === 0}
      <EmptyState icon="trash-2" title={t('trash.noItems')} hint={t('trash.noItemsHint')} />
    {:else}
      <ul class="list">
        {#each app.trash as item (item.id)}
          <li>
            <button class="row" onclick={() => (selected = item)}>
              <span class="icon"><Icon name={item.isDir ? 'folder' : 'file'} size={20} /></span>
              <span class="text">
                <span class="name">{item.name}</span>
                <span class="meta">
                  {t('trash.from')}: {item.locationLabel || item.originalParent}
                </span>
                <span class="meta">{formatRelative(item.deletedAt)}</span>
              </span>
              <Icon name="chevron-right" size={18} class="chev" />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

{#if selected}
  <ActionSheet
    title={selected.name}
    actions={[
      { id: 'restore', label: t('trash.restore'), icon: 'undo-2' },
      { id: 'purge', label: t('trash.deleteForever'), icon: 'trash-2', danger: true },
    ]}
    onselect={(id) => {
      if (!selected) return;
      if (id === 'restore') void restore(selected);
      else void purge(selected);
    }}
    onclose={() => (selected = null)}
  />
{/if}

<style>
  .appbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
    border-bottom: 1px solid var(--line);
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  .titles h1 {
    font-family: var(--font-display);
    font-size: 1.25rem;
  }
  .titles span {
    font-size: 0.75rem;
    color: var(--ink-3);
  }
  .btn.small {
    min-height: 40px;
    padding: 0 var(--space-3);
    font-size: 0.8125rem;
  }
  .list {
    padding: var(--space-2) var(--space-2) calc(var(--safe-bottom) + var(--space-6));
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-md);
    text-align: left;
  }
  .row:active {
    background: var(--paper-sunken);
  }
  .icon {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: var(--paper-sunken);
    color: var(--ink-3);
    flex: none;
  }
  .text {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 1px;
  }
  .name {
    font-size: 0.9375rem;
    font-weight: 550;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    font-size: 0.6875rem;
    color: var(--ink-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.chev) {
    color: var(--ink-3);
  }
</style>
