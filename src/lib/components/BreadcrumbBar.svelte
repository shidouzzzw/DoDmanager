<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import type { Crumb } from '../types';
  import { middleEllipsis } from '../services/format';
  import Icon from './Icon.svelte';

  interface Props {
    crumbs: Crumb[];
    onnavigate: (crumb: Crumb, index: number) => void;
  }

  let { crumbs, onnavigate }: Props = $props();
  let scroller = $state<HTMLElement | null>(null);

  $effect(() => {
    // Yeni klasöre girildiğinde son kırıntı görünür olsun.
    const length = crumbs.length;
    if (length && scroller) {
      requestAnimationFrame(() => {
        scroller?.scrollTo({ left: scroller.scrollWidth, behavior: 'smooth' });
      });
    }
  });
</script>

<nav class="crumbs" bind:this={scroller} aria-label={t('props.path')}>
  {#each crumbs as crumb, index (crumb.path)}
    {#if index > 0}
      <Icon name="chevron-right" size={14} class="sep" />
    {/if}
    <button
      class="crumb"
      class:current={index === crumbs.length - 1}
      aria-current={index === crumbs.length - 1 ? 'page' : undefined}
      onclick={() => onnavigate(crumb, index)}
    >
      {#if index === 0}
        <Icon name="folder" size={14} />
      {/if}
      {middleEllipsis(crumb.name, 22)}
    </button>
  {/each}
</nav>

<style>
  .crumbs {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 var(--space-4) var(--space-2);
    overflow-x: auto;
    scrollbar-width: none;
    white-space: nowrap;
  }
  .crumbs::-webkit-scrollbar {
    display: none;
  }
  .crumb {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 36px;
    padding: 0 var(--space-2);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-size: 0.8125rem;
    font-weight: 520;
    flex: none;
  }
  .crumb.current {
    color: var(--ink);
    background: var(--paper-sunken);
  }
  .crumbs :global(.sep) {
    color: var(--line-strong);
    flex: none;
  }
</style>
