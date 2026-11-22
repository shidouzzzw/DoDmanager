<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    icon?: string;
    title: string;
    hint?: string;
    action?: Snippet;
  }

  let { icon = 'folder-open', title, hint, action }: Props = $props();
</script>

<div class="empty-state">
  <div class="art" aria-hidden="true">
    <Icon name={icon} size={36} stroke={1.5} />
  </div>
  <h3>{title}</h3>
  {#if hint}<p>{hint}</p>{/if}
  {#if action}
    <div class="action">{@render action()}</div>
  {/if}
</div>

<style>
  .action {
    margin-top: var(--space-2);
  }
</style>
