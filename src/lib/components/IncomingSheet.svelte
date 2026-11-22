<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import type { IncomingFile } from '../types';
  import BottomSheet from './BottomSheet.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    files: IncomingFile[];
    onsave: () => void;
    ondismiss: () => void;
  }

  let { files, onsave, ondismiss }: Props = $props();
</script>

<BottomSheet title={t('incoming.title')} onclose={ondismiss}>
  <div class="body">
    <p>{t('incoming.body', { count: files.length })}</p>
    <ul>
      {#each files.slice(0, 6) as file (file.uri)}
        <li>
          <Icon name="file" size={18} />
          <span>{file.name}</span>
        </li>
      {/each}
      {#if files.length > 6}
        <li class="more">+{files.length - 6}</li>
      {/if}
    </ul>
  </div>
  <div class="actions">
    <button class="btn btn-secondary" onclick={ondismiss}>{t('incoming.dismiss')}</button>
    <button class="btn btn-primary" onclick={onsave}>{t('incoming.save')}</button>
  </div>
</BottomSheet>

<style>
  .body {
    padding: 0 var(--space-5) var(--space-3);
    display: grid;
    gap: var(--space-3);
    color: var(--ink-2);
    font-size: 0.875rem;
  }
  ul {
    display: grid;
    gap: var(--space-2);
    max-height: 200px;
    overflow-y: auto;
  }
  li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--ink);
    font-size: 0.8125rem;
  }
  li span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .more {
    color: var(--ink-3);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
  }
</style>
