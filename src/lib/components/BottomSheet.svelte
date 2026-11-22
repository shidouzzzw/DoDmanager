<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
    labelledBy?: string;
  }

  let { title, onclose, children, footer }: Props = $props();
</script>

<button class="scrim" aria-label={t('common.close')} onclick={onclose}></button>
<div class="sheet" role="dialog" aria-modal="true" aria-label={title ?? t('common.more')}>
  <div class="sheet-grabber" aria-hidden="true"></div>
  {#if title}
    <h2 class="sheet-title">{title}</h2>
  {/if}
  <div class="sheet-body">
    {@render children()}
  </div>
  {#if footer}
    <div class="sheet-footer">
      {@render footer()}
    </div>
  {/if}
</div>

<style>
  .sheet-body {
    display: flex;
    flex-direction: column;
    padding-bottom: var(--space-2);
  }
  .sheet-footer {
    padding: var(--space-3) var(--space-5) var(--space-2);
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    border-top: 1px solid var(--line);
    margin-top: var(--space-2);
  }
</style>
