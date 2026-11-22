<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import Icon from './Icon.svelte';

  interface Props {
    count: number;
    onshare: () => void;
    ondelete: () => void;
    onmore: () => void;
    onclear: () => void;
  }

  let { count, onshare, ondelete, onmore, onclear }: Props = $props();
</script>

<div class="bar kb-aware" role="toolbar" aria-label={t('browser.selectionMode')}>
  <span class="count" aria-live="polite">
    <button class="icon-btn" aria-label={t('common.deselectAll')} onclick={onclear}>
      <Icon name="x" size={20} />
    </button>
    {t('browser.selected', { count })}
  </span>
  <div class="actions">
    <button class="icon-btn" aria-label={t('common.share')} onclick={onshare}>
      <Icon name="share-2" size={21} />
    </button>
    <button class="icon-btn danger" aria-label={t('common.delete')} onclick={ondelete}>
      <Icon name="trash-2" size={21} />
    </button>
    <button class="icon-btn" aria-label={t('a11y.more')} onclick={onmore}>
      <Icon name="more-vertical" size={21} />
    </button>
  </div>
</div>

<style>
  .bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-2) max(var(--space-2), var(--safe-right)) var(--space-2)
      max(var(--space-2), var(--safe-left));
    background: var(--paper-raised);
    border-top: 1px solid var(--line);
    box-shadow: var(--shadow-md);
    animation: rise var(--dur-base) var(--ease-spring);
  }
  .count {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 620;
    padding-left: var(--space-1);
    white-space: nowrap;
  }
  .actions {
    display: flex;
    gap: var(--space-1);
  }
  .danger {
    color: var(--danger);
  }
</style>
