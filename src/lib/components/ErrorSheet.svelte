<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { errorActionLabel, type AppError } from '../services/errors';
  import BottomSheet from './BottomSheet.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    error: AppError;
    onclose: () => void;
    onaction: (action: AppError['action']) => void;
  }

  let { error, onclose, onaction }: Props = $props();

  const actionLabel = $derived(errorActionLabel(error.action));
</script>

<BottomSheet title={t('errors.title')} {onclose}>
  <div class="body">
    <div class="mark" aria-hidden="true"><Icon name="triangle-alert" size={26} /></div>
    <p class="message">{error.localized()}</p>
    {#if error.detail}
      <p class="detail">{error.detail}</p>
    {/if}
  </div>
  <div class="actions">
    {#if error.action !== 'none'}
      <button class="btn btn-primary btn-block" onclick={() => onaction(error.action)}>
        {actionLabel}
      </button>
    {/if}
    <button class="btn btn-secondary btn-block" onclick={onclose}>{t('common.close')}</button>
  </div>
</BottomSheet>

<style>
  .body {
    display: flex;
    gap: var(--space-3);
    padding: 0 var(--space-5) var(--space-3);
    align-items: flex-start;
  }
  .mark {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    display: grid;
    place-items: center;
    background: var(--danger-soft);
    color: var(--danger);
  }
  .message {
    font-size: 0.9375rem;
    line-height: 1.55;
    color: var(--ink-2);
  }
  .detail {
    margin-top: var(--space-2);
    font-size: 0.75rem;
    color: var(--ink-3);
    font-family: var(--font-mono);
    word-break: break-word;
  }
  .actions {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5) var(--space-3);
  }
</style>
