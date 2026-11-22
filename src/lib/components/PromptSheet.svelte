<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import BottomSheet from './BottomSheet.svelte';

  interface Props {
    title: string;
    label: string;
    initial?: string;
    confirmLabel?: string;
    busy?: boolean;
    onclose: () => void;
    onconfirm: (value: string) => void;
  }

  let {
    title,
    label,
    initial = '',
    confirmLabel = t('common.save'),
    busy = false,
    onclose,
    onconfirm,
  }: Props = $props();

  let value = $state(initial);
  let input = $state<HTMLInputElement | null>(null);
  let error = $state('');

  $effect(() => {
    input?.focus();
    input?.setSelectionRange(0, value.length);
  });

  function submit(event?: SubmitEvent) {
    event?.preventDefault();
    const trimmed = value.trim();
    if (!trimmed) {
      error = t('errors.INVALID_NAME');
      return;
    }
    if (trimmed.includes('/') || trimmed === '.' || trimmed === '..') {
      error = t('errors.INVALID_NAME');
      return;
    }
    error = '';
    onconfirm(trimmed);
  }
</script>

<BottomSheet {title} {onclose}>
  <form class="form" onsubmit={submit}>
    <div class="field">
      <label for="prompt-input">{label}</label>
      <input
        id="prompt-input"
        class="input"
        bind:this={input}
        bind:value
        autocomplete="off"
        autocapitalize="none"
        spellcheck="false"
        aria-invalid={!!error}
        aria-describedby={error ? 'prompt-error' : undefined}
      />
      {#if error}<p class="error" id="prompt-error">{error}</p>{/if}
    </div>
    <div class="actions">
      <button type="button" class="btn btn-secondary" onclick={onclose}>{t('common.cancel')}</button
      >
      <button type="submit" class="btn btn-primary" disabled={busy}>
        {busy ? t('common.loading') : confirmLabel}
      </button>
    </div>
  </form>
</BottomSheet>

<style>
  .form {
    display: grid;
    gap: var(--space-3);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-5) var(--space-3);
  }
  .error {
    color: var(--danger);
    font-size: 0.8125rem;
  }
</style>
