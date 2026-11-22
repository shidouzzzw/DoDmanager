<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import Icon from './Icon.svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    message: string;
    undo?: (() => Promise<void>) | null;
    onundo: () => void;
    onclose: () => void;
    leading?: Snippet;
    offset?: number;
  }

  let { message, undo, onundo, onclose, offset = 0 }: Props = $props();
</script>

<div class="snackbar" role="status" aria-live="polite" style="--snackbar-offset: {offset}px">
  <span class="message">{message}</span>
  {#if undo}
    <button class="action" onclick={onundo}>{t('snackbar.undo')}</button>
  {/if}
  <button class="icon-btn close" aria-label={t('common.close')} onclick={onclose}>
    <Icon name="x" size={18} />
  </button>
</div>

<style>
  .snackbar {
    position: fixed;
    left: max(var(--space-3), var(--safe-left));
    right: max(var(--space-3), var(--safe-right));
    bottom: calc(var(--snackbar-offset) + var(--safe-bottom) + var(--space-3));
    z-index: 35;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 56px;
    padding: var(--space-2) var(--space-2) var(--space-2) var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--ink);
    color: var(--paper);
    box-shadow: var(--shadow-lg);
    animation: rise var(--dur-base) var(--ease-spring);
    transition: bottom var(--dur-base) var(--ease-spring);
  }
  .message {
    flex: 1;
    font-size: 0.875rem;
    line-height: 1.35;
  }
  .action {
    min-height: 40px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-pill);
    color: var(--paper);
    font-weight: 650;
    font-size: 0.875rem;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .close {
    color: var(--paper);
    opacity: 0.7;
    width: 40px;
    height: 40px;
    min-width: 40px;
    min-height: 40px;
  }
</style>
