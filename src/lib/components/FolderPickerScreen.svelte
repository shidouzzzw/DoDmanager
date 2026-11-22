<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '../i18n';
  import { listDir, newJobId, copyEntries, moveEntries, createDir } from '../services/files';
  import { app } from '../state/app.svelte';
  import type { DirListing, FileEntry, Location } from '../types';
  import { suggestName } from '../ui/fileMeta';
  import EmptyState from './EmptyState.svelte';
  import Icon from './Icon.svelte';
  import PromptSheet from './PromptSheet.svelte';

  let stack = $state<{ name: string; path: string }[]>([]);
  let listing = $state<DirListing | null>(null);
  let loading = $state(true);
  let busy = $state(false);
  let showCreate = $state(false);
  let error = $state<string | null>(null);

  const intent = $derived(app.picker);
  const currentPath = $derived(stack.length ? stack[stack.length - 1].path : '');
  const folders = $derived((listing?.entries ?? []).filter((entry) => entry.isDir));

  onMount(async () => {
    const base: Location | null =
      app.currentLocation ?? app.access?.defaultLocation ?? app.locations[0] ?? null;
    const path = base?.path;
    if (!path) {
      loading = false;
      return;
    }
    stack = [{ name: base?.label ?? t('common.folder'), path }];
    await load(path);
  });

  async function load(path: string) {
    loading = true;
    error = null;
    try {
      listing = await listDir(path);
    } catch (e) {
      const failure = app.fail(e);
      error = failure.localized();
      listing = null;
    } finally {
      loading = false;
    }
  }

  async function enter(entry: FileEntry) {
    stack = [...stack, { name: entry.name, path: entry.path }];
    await load(entry.path);
  }

  async function goBackTo(index: number) {
    stack = stack.slice(0, index + 1);
    await load(currentPath);
  }

  function isInsideSource(destination: string): boolean {
    if (!intent) return false;
    return intent.paths.some(
      (source) => destination === source || destination.startsWith(`${source}/`),
    );
  }

  async function confirm() {
    if (!intent || !currentPath) return;
    if (isInsideSource(currentPath)) {
      app.fail({ code: 'INVALID_NAME' });
      return;
    }
    busy = true;
    error = null;
    try {
      const command = intent.mode === 'copy' ? copyEntries : moveEntries;
      const moved = await command(newJobId(), intent.paths, currentPath);
      const fromShare =
        app.incoming.length > 0 && intent.paths.some((path) => path.startsWith('content://'));
      if (fromShare) app.dismissIncoming();
      app.toast(
        intent.mode === 'copy'
          ? t('snackbar.copied', { count: moved.length })
          : t('snackbar.moved', { count: moved.length }),
      );
      const back = intent.returnScreen;
      app.picker = null;
      app.screen = back;
      if (back === 'browser') await app.reload();
      else await app.refreshHome();
    } catch (e) {
      app.fail(e);
    } finally {
      busy = false;
    }
  }

  async function createFolder(name: string) {
    try {
      const folder = await createDir(currentPath, name);
      showCreate = false;
      await load(currentPath);
      await enter({ ...folder, isDir: true } as FileEntry);
    } catch (e) {
      app.fail(e);
    }
  }
</script>

<div class="screen">
  <header class="appbar pad-x">
    <button class="icon-btn" aria-label={t('a11y.back')} onclick={() => app.back()}>
      <Icon name="chevron-left" size={24} />
    </button>
    <div class="titles">
      <h1>{intent?.mode === 'move' ? t('transfer.moveTo') : t('transfer.copyTo')}</h1>
      <span>{t('transfer.title')}</span>
    </div>
    <button class="icon-btn" aria-label={t('common.refresh')} onclick={() => load(currentPath)}>
      <Icon name="refresh-cw" size={20} />
    </button>
  </header>

  <div class="crumbs">
    {#each stack as crumb, index (crumb.path)}
      <button
        class="chip"
        class:is-active={index === stack.length - 1}
        onclick={() => goBackTo(index)}
      >
        {crumb.name}
      </button>
    {/each}
  </div>

  <div class="list">
    {#if loading}
      {#each Array(6) as _, index (index)}
        <div class="row skeleton-row">
          <div class="skeleton" style="width: 40px; height: 40px"></div>
          <div class="skeleton" style="width: 60%; height: 14px"></div>
        </div>
      {/each}
    {:else if error}
      <EmptyState icon="circle-alert" title={error} />
    {:else if folders.length === 0}
      <EmptyState
        icon="folder"
        title={t('browser.emptyFolder')}
        hint={t('browser.emptyFolderHint')}
      >
        {#snippet action()}
          <button class="btn btn-secondary" onclick={() => (showCreate = true)}>
            <Icon name="folder-plus" size={18} />
            {t('transfer.createFolderHere')}
          </button>
        {/snippet}
      </EmptyState>
    {:else}
      {#each folders as entry (entry.path)}
        <button class="row" onclick={() => enter(entry)}>
          <span class="icon"><Icon name="folder" size={20} /></span>
          <span class="name">{entry.name}</span>
          <Icon name="chevron-right" size={18} class="chev" />
        </button>
      {/each}
    {/if}
  </div>

  <footer class="footer kb-aware">
    <div class="here">
      <Icon name="folder-input" size={18} />
      <span>{stack.length ? stack[stack.length - 1].name : t('common.none')}</span>
    </div>
    <div class="buttons">
      <button class="btn btn-secondary" onclick={() => (showCreate = true)} disabled={busy}>
        <Icon name="folder-plus" size={18} />
        {t('browser.createFolder')}
      </button>
      <button class="btn btn-primary" onclick={confirm} disabled={busy || !currentPath}>
        {busy
          ? t('common.loading')
          : intent?.mode === 'move'
            ? t('transfer.moveHere')
            : t('transfer.copyHere')}
      </button>
    </div>
  </footer>
</div>

{#if showCreate}
  <PromptSheet
    title={t('browser.createFolder')}
    label={t('browser.folderName')}
    initial={suggestName(listing?.entries ?? [], 'Yeni klasör')}
    onclose={() => (showCreate = false)}
    onconfirm={createFolder}
  />
{/if}

<style>
  .appbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  .titles h1 {
    font-family: var(--font-display);
    font-size: 1.125rem;
  }
  .titles span {
    font-size: 0.6875rem;
    color: var(--ink-3);
  }
  .crumbs {
    display: flex;
    gap: var(--space-2);
    overflow-x: auto;
    scrollbar-width: none;
    padding: 0 var(--space-4) var(--space-3);
  }
  .crumbs::-webkit-scrollbar {
    display: none;
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--space-2);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    min-height: 60px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-md);
    text-align: left;
  }
  .row:active {
    background: var(--paper-sunken);
  }
  .icon {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .name {
    flex: 1;
    font-size: 0.9375rem;
    font-weight: 540;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.chev) {
    color: var(--ink-3);
  }
  .skeleton-row {
    gap: var(--space-3);
  }
  .footer {
    position: sticky;
    bottom: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4) calc(var(--safe-bottom) + var(--space-3));
    background: var(--paper-raised);
    border-top: 1px solid var(--line);
  }
  .here {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.8125rem;
    color: var(--ink-2);
  }
  .buttons {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-2);
  }
</style>
