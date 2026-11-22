<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { assetUrl, thumbnail } from '../services/files';
  import { formatSize } from '../services/format';
  import type { FileEntry } from '../types';
  import { categoryColorVar, categoryIcon } from '../ui/fileMeta';
  import Icon from './Icon.svelte';

  interface Props {
    entry: FileEntry;
    view: 'grid' | 'list';
    index: number;
    selected: boolean;
    selecting: boolean;
    dropTarget: boolean;
    onactivate: (entry: FileEntry, index: number) => void;
    onlongpress: (entry: FileEntry, index: number) => void;
    onhover: (index: number) => void;
    onpointerdownitem: (event: PointerEvent, entry: FileEntry, index: number) => void;
    onpointermoveitem: (event: PointerEvent) => void;
    onpointerupitem: (event: PointerEvent) => void;
  }

  let {
    entry,
    view,
    index,
    selected,
    selecting,
    dropTarget,
    onactivate,
    onlongpress,
    onhover,
    onpointerdownitem,
    onpointermoveitem,
    onpointerupitem,
  }: Props = $props();

  let thumb = $state<string | null>(null);
  let longPressed = false;
  let pressTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const path = entry.path;
    const isImage = entry.category === 'image' && !entry.isDir;
    thumb = null;
    if (!isImage) return;
    let cancelled = false;
    thumbnail(path, view === 'grid' ? 320 : 96)
      .then((result) => {
        if (!cancelled) thumb = assetUrl(result.path);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  function clearTimer() {
    if (pressTimer) clearTimeout(pressTimer);
    pressTimer = null;
  }

  function handlePointerDown(event: PointerEvent) {
    longPressed = false;
    clearTimer();
    onpointerdownitem(event, entry, index);
    if (event.pointerType === 'mouse' && event.button !== 0) return;
    pressTimer = setTimeout(() => {
      longPressed = true;
      onlongpress(entry, index);
    }, 420);
  }

  function handlePointerUp(event: PointerEvent) {
    clearTimer();
    onpointerupitem(event);
  }

  function handleClick() {
    if (longPressed) {
      longPressed = false;
      return;
    }
    if (selecting) {
      onlongpress(entry, index);
      return;
    }
    onactivate(entry, index);
  }

  const subtitle = $derived(
    entry.isDir
      ? entry.childCount != null
        ? t('browser.folderItems', { count: entry.childCount })
        : t('common.folder')
      : formatSize(entry.size),
  );
</script>

{#if view === 'grid'}
  <button
    class="tile"
    class:selected
    class:drop={dropTarget}
    data-dir-path={entry.isDir ? entry.path : undefined}
    aria-pressed={selecting ? selected : undefined}
    aria-label={t('a11y.item', {
      name: entry.name,
      type: entry.isDir ? t('common.folder') : t('common.file'),
      size: formatSize(entry.size),
    })}
    onclick={handleClick}
    onpointerdown={handlePointerDown}
    onpointermove={onpointermoveitem}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerUp}
    onpointerenter={() => onhover(index)}
    oncontextmenu={(event) => event.preventDefault()}
  >
    <span class="thumb" style="--tint: {categoryColorVar(entry)}">
      {#if thumb}
        <img src={thumb} alt="" loading="lazy" decoding="async" />
      {:else}
        <Icon name={categoryIcon(entry)} size={30} stroke={1.5} />
      {/if}
      {#if entry.isDir}
        <span class="tab" aria-hidden="true"></span>
      {/if}
    </span>
    <span class="meta">
      <span class="name">{entry.name}</span>
      <span class="sub">{subtitle}</span>
    </span>
    {#if selected}
      <span class="check" aria-hidden="true"><Icon name="check" size={14} stroke={2.4} /></span>
    {/if}
  </button>
{:else}
  <button
    class="row"
    class:selected
    class:drop={dropTarget}
    data-dir-path={entry.isDir ? entry.path : undefined}
    aria-pressed={selecting ? selected : undefined}
    aria-label={t('a11y.item', {
      name: entry.name,
      type: entry.isDir ? t('common.folder') : t('common.file'),
      size: formatSize(entry.size),
    })}
    onclick={handleClick}
    onpointerdown={handlePointerDown}
    onpointermove={onpointermoveitem}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerUp}
    onpointerenter={() => onhover(index)}
    oncontextmenu={(event) => event.preventDefault()}
  >
    <span class="icon" style="--tint: {categoryColorVar(entry)}">
      {#if thumb}
        <img src={thumb} alt="" loading="lazy" decoding="async" />
      {:else}
        <Icon name={categoryIcon(entry)} size={20} stroke={1.6} />
      {/if}
    </span>
    <span class="meta">
      <span class="name">{entry.name}</span>
      <span class="sub">{subtitle}</span>
    </span>
    {#if selected}
      <span class="check" aria-hidden="true"><Icon name="check" size={14} stroke={2.4} /></span>
    {:else if entry.isDir}
      <Icon name="chevron-right" size={18} class="chev" />
    {/if}
  </button>
{/if}

<style>
  .tile {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: 100%;
    height: 100%;
    padding: 0;
    text-align: left;
    border-radius: var(--radius-md);
    transition:
      transform var(--dur-fast) var(--ease-spring),
      background-color var(--dur-fast) var(--ease-out);
  }
  .tile:active {
    transform: scale(0.965);
  }
  .tile.selected {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .tile.drop {
    box-shadow: inset 0 0 0 2px var(--accent);
    background: var(--accent-soft);
  }
  .thumb {
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
    aspect-ratio: 1 / 1;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--tint) 12%, var(--paper-raised));
    color: var(--tint);
    overflow: hidden;
    border: 1px solid var(--line);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  /* Klasörler için dolap çekmecesi izi. */
  .tab {
    position: absolute;
    top: 0;
    left: 12%;
    width: 30%;
    height: 6px;
    border-radius: 0 0 4px 4px;
    background: color-mix(in srgb, var(--tint) 45%, transparent);
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 1px;
    padding: 0 2px;
  }
  .name {
    font-size: 0.8125rem;
    font-weight: 560;
    line-height: 1.25;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 0.6875rem;
    color: var(--ink-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .check {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--on-accent);
    box-shadow: var(--shadow-sm);
    animation: pop var(--dur-fast) var(--ease-spring);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    height: 100%;
    padding: 0 var(--space-4);
    text-align: left;
    border-radius: var(--radius-md);
  }
  .row:active {
    background: var(--paper-sunken);
  }
  .row.selected {
    background: var(--accent-soft);
  }
  .row.drop {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .icon {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--tint) 12%, var(--paper-raised));
    color: var(--tint);
    overflow: hidden;
  }
  .icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .row .meta {
    flex: 1;
  }
  .row .name {
    font-size: 0.9375rem;
  }
  :global(.chev) {
    color: var(--ink-3);
    flex: none;
  }
</style>
