<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { formatRelative, formatSize } from '../services/format';
  import { app } from '../state/app.svelte';
  import type { Location, SavedPath } from '../types';
  import Icon from './Icon.svelte';
  import StorageMeter from './StorageMeter.svelte';

  let scrollEl = $state<HTMLDivElement | null>(null);
  let scrolled = $state(false);

  function onScroll() {
    scrolled = (scrollEl?.scrollTop ?? 0) > 24;
  }

  async function openLocation(location: Location) {
    await app.openLocation(location);
  }

  async function addLocation() {
    const location = await app.pickNewLocation();
    if (location) app.toast(t('onboarding.scopedReady'));
  }

  function openFavorite(item: SavedPath) {
    const location: Location = {
      id: `fav-${item.path}`,
      label: item.name,
      path: item.path,
      provider: item.path.startsWith('content://') ? 'saf' : 'local',
      removable: false,
      addedAt: item.touchedAt,
    };
    void app.openLocation(location);
  }

  function openRecent(item: SavedPath) {
    app.openPreview(
      [
        {
          path: item.path,
          name: item.name,
          isDir: item.isDir,
          size: 0,
          modified: item.touchedAt,
          mime: item.mime,
          category: 'other',
          readOnly: false,
        },
      ],
      0,
    );
  }

  const accessLabel = $derived(
    app.settings.accessMode === 'all_files' ? t('home.accessAllFiles') : t('home.accessScoped'),
  );

  const needsFolder = $derived(
    app.settings.accessMode === 'scoped' && !app.locations.some((l) => l.provider === 'saf'),
  );
</script>

<div class="screen home">
  <header class="appbar pad-x" class:raised={scrolled}>
    <span class="brand" aria-hidden="true">D</span>
    <span class="title" class:visible={scrolled}>{t('home.title')}</span>
    <button class="chip access" onclick={() => app.goToSettings()}>
      <Icon
        name={app.settings.accessMode === 'all_files' ? 'hard-drive' : 'shield-check'}
        size={14}
      />
      {accessLabel}
    </button>
    <button class="icon-btn" aria-label={t('a11y.settings')} onclick={() => app.goToSettings()}>
      <Icon name="settings" size={22} />
    </button>
  </header>

  <div class="scroll" bind:this={scrollEl} onscroll={onScroll}>
    <section class="intro pad-x">
      <h1>{t('home.title')}</h1>
      <p>{t('app.tagline')}</p>
    </section>

    <StorageMeter storage={app.storage} loading={!app.ready} />

    {#if needsFolder}
      <button class="notice pad-x" onclick={addLocation}>
        <span class="notice-icon"><Icon name="folder-plus" size={20} /></span>
        <span class="notice-text">
          <strong>{t('onboarding.pickFolder')}</strong>
          <small>{t('onboarding.scopedBody')}</small>
        </span>
        <Icon name="chevron-right" size={18} />
      </button>
    {/if}

    <section class="block">
      <div class="block-head pad-x">
        <h2 class="section-title">{t('home.locations')}</h2>
        <button class="text-btn" onclick={addLocation}>
          <Icon name="plus" size={16} />
          {t('home.addLocation')}
        </button>
      </div>
      <div class="locations">
        {#each app.locations as location (location.id)}
          <button class="loc-card" onclick={() => openLocation(location)}>
            <span class="loc-icon">
              <Icon name={location.removable ? 'hard-drive' : 'folder'} size={24} stroke={1.5} />
            </span>
            <span class="loc-label">{location.label}</span>
            <span class="loc-path">{location.provider === 'saf' ? 'SAF' : location.path}</span>
          </button>
        {/each}
        <button class="loc-card add" onclick={addLocation}>
          <span class="loc-icon"><Icon name="plus" size={24} /></span>
          <span class="loc-label">{t('home.addLocation')}</span>
          <span class="loc-path">{t('onboarding.scopedTitle')}</span>
        </button>
      </div>
    </section>

    <section class="block">
      <div class="block-head pad-x">
        <h2 class="section-title">{t('home.favorites')}</h2>
        <button
          class="icon-btn small"
          aria-label={t('common.refresh')}
          onclick={() => app.refreshFavorites()}
        >
          <Icon name="refresh-cw" size={16} />
        </button>
      </div>
      {#if app.favorites.length === 0}
        <div class="inline-empty pad-x">
          <Icon name="star" size={18} />
          <span>
            <strong>{t('home.noFavorites')}</strong>
            <small>{t('home.noFavoritesHint')}</small>
          </span>
        </div>
      {:else}
        <ul class="list pad-x">
          {#each app.favorites as favorite (favorite.path)}
            <li>
              <button class="row" onclick={() => openFavorite(favorite)}>
                <span class="row-icon"><Icon name="folder" size={20} /></span>
                <span class="row-text">
                  <span class="row-title">{favorite.name}</span>
                  <span class="row-sub"
                    >{favorite.path.startsWith('content://') ? 'SAF' : favorite.path}</span
                  >
                </span>
                <Icon name="chevron-right" size={18} class="chev" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="block">
      <div class="block-head pad-x">
        <h2 class="section-title">{t('home.recents')}</h2>
      </div>
      {#if app.recents.length === 0}
        <div class="inline-empty pad-x">
          <Icon name="refresh-cw" size={18} />
          <span>
            <strong>{t('home.noRecents')}</strong>
            <small>{t('home.noRecentsHint')}</small>
          </span>
        </div>
      {:else}
        <div class="recents">
          {#each app.recents.slice(0, 8) as recent (recent.path)}
            <button class="recent" onclick={() => openRecent(recent)}>
              <span class="recent-icon">
                <Icon name={recent.isDir ? 'folder' : 'file'} size={20} />
              </span>
              <span class="recent-name">{recent.name}</span>
              <span class="recent-time">{formatRelative(recent.touchedAt)}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <button class="trash-card" onclick={() => app.goToTrash()}>
      <span class="trash-icon"><Icon name="trash-2" size={22} /></span>
      <span class="trash-text">
        <strong>{t('home.trash')}</strong>
        <small>
          {#if app.trash.length}
            {t('home.trashCount', { count: app.trash.length, size: formatSize(app.trashBytes) })}
          {:else}
            {t('home.trashEmpty')}
          {/if}
        </small>
      </span>
      <Icon name="chevron-right" size={18} class="chev" />
    </button>

    <footer class="foot pad-x">
      <Icon name="wifi-off" size={16} />
      <span>{t('settings.privacyBody')}</span>
    </footer>
  </div>
</div>

<style>
  .home {
    background:
      radial-gradient(120% 55% at 12% -8%, var(--accent-soft), transparent 60%), var(--paper);
  }
  .appbar {
    position: sticky;
    top: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
    background: color-mix(in srgb, var(--paper) 88%, transparent);
    backdrop-filter: blur(14px);
    transition: box-shadow var(--dur-fast) var(--ease-out);
  }
  .appbar.raised {
    box-shadow: 0 1px 0 var(--line);
  }
  .brand {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    background: var(--ink);
    color: var(--paper);
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 0.9375rem;
  }
  .title {
    flex: 1;
    font-family: var(--font-display);
    font-weight: 640;
    font-size: 1.0625rem;
    opacity: 0;
    transform: translateY(6px);
    transition:
      opacity var(--dur-fast) var(--ease-out),
      transform var(--dur-fast) var(--ease-out);
  }
  .title.visible {
    opacity: 1;
    transform: none;
  }
  .access {
    max-width: 44vw;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .intro {
    padding-top: var(--space-3);
    padding-bottom: var(--space-5);
  }
  .intro h1 {
    font-size: clamp(2.25rem, 9vw, 3rem);
    line-height: 1.02;
    letter-spacing: -0.035em;
    font-weight: 680;
  }
  .intro p {
    margin-top: var(--space-2);
    color: var(--ink-3);
    font-size: 0.9375rem;
    max-width: 34ch;
  }
  .notice {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: calc(100% - var(--space-8));
    margin: 0 var(--space-4) var(--space-5);
    padding: var(--space-3);
    border-radius: var(--radius-lg);
    background: var(--accent-soft);
    color: var(--accent);
    text-align: left;
  }
  .notice-icon {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: var(--paper-raised);
    flex: none;
  }
  .notice-text {
    flex: 1;
    display: grid;
    gap: 2px;
  }
  .notice-text strong {
    font-size: 0.9375rem;
    color: var(--ink);
  }
  .notice-text small {
    font-size: 0.75rem;
    color: var(--ink-2);
    line-height: 1.4;
  }
  .block {
    margin-bottom: var(--space-5);
  }
  .block-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-2);
  }
  .text-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    min-height: 40px;
    padding: 0 var(--space-2);
    color: var(--accent);
    font-size: 0.8125rem;
    font-weight: 600;
  }
  .locations {
    display: flex;
    gap: var(--space-2);
    padding: 0 var(--space-4) var(--space-1);
    overflow-x: auto;
    scrollbar-width: none;
  }
  .locations::-webkit-scrollbar {
    display: none;
  }
  .loc-card {
    flex: none;
    width: 148px;
    height: 132px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-end;
    gap: 4px;
    padding: var(--space-3);
    border-radius: var(--radius-lg);
    background: var(--paper-raised);
    border: 1px solid var(--line);
    box-shadow: var(--shadow-sm);
    text-align: left;
    transition: transform var(--dur-fast) var(--ease-spring);
  }
  .loc-card:active {
    transform: scale(0.97);
  }
  .loc-icon {
    width: 44px;
    height: 44px;
    border-radius: 13px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
    margin-bottom: auto;
  }
  .loc-label {
    font-weight: 600;
    font-size: 0.875rem;
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .loc-path {
    font-size: 0.6875rem;
    color: var(--ink-3);
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .loc-card.add {
    background: transparent;
    border-style: dashed;
    color: var(--ink-3);
    box-shadow: none;
  }
  .loc-card.add .loc-icon {
    background: var(--paper-sunken);
    color: var(--ink-3);
  }
  .inline-empty {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    padding-top: var(--space-2);
    padding-bottom: var(--space-3);
    color: var(--ink-3);
  }
  .inline-empty strong {
    display: block;
    color: var(--ink-2);
    font-size: 0.875rem;
    font-weight: 600;
  }
  .inline-empty small {
    font-size: 0.75rem;
  }
  .list {
    display: grid;
    gap: 2px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) 0;
    text-align: left;
    border-radius: var(--radius-md);
  }
  .row:active {
    background: var(--paper-sunken);
  }
  .row-icon {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .row-text {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 1px;
  }
  .row-title {
    font-size: 0.9375rem;
    font-weight: 550;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row-sub {
    font-size: 0.6875rem;
    color: var(--ink-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.chev) {
    color: var(--ink-3);
    flex: none;
  }
  .recents {
    display: flex;
    gap: var(--space-2);
    padding: 0 var(--space-4);
    overflow-x: auto;
    scrollbar-width: none;
  }
  .recents::-webkit-scrollbar {
    display: none;
  }
  .recent {
    flex: none;
    width: 124px;
    height: 104px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--paper-raised);
    border: 1px solid var(--line);
    text-align: left;
  }
  .recent-icon {
    color: var(--accent);
    margin-bottom: auto;
  }
  .recent-name {
    font-size: 0.75rem;
    font-weight: 550;
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recent-time {
    font-size: 0.6875rem;
    color: var(--ink-3);
  }
  .trash-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: calc(100% - var(--space-8));
    margin: 0 var(--space-4) var(--space-5);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--paper-raised);
    border: 1px solid var(--line);
    text-align: left;
  }
  .trash-icon {
    width: 44px;
    height: 44px;
    border-radius: 13px;
    display: grid;
    place-items: center;
    background: var(--danger-soft);
    color: var(--danger);
    flex: none;
  }
  .trash-text {
    flex: 1;
    display: grid;
    gap: 2px;
  }
  .trash-text strong {
    font-size: 0.9375rem;
    font-weight: 600;
  }
  .trash-text small {
    font-size: 0.75rem;
    color: var(--ink-3);
  }
  .foot {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    padding-bottom: calc(var(--safe-bottom) + var(--space-6));
    color: var(--ink-3);
    font-size: 0.6875rem;
    line-height: 1.5;
  }
  .foot span {
    max-width: 38ch;
  }
  .icon-btn.small {
    width: 40px;
    height: 40px;
    min-width: 40px;
    min-height: 40px;
  }
</style>
