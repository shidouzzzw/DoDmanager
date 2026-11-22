<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { localeTag, t } from '../i18n';
  import { formatDate, formatSize } from '../services/format';
  import type { FileEntry } from '../types';
  import { categoryLabelKey, categoryIcon } from '../ui/fileMeta';
  import BottomSheet from './BottomSheet.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    entry: FileEntry;
    onclose: () => void;
    onopenwith: () => void;
  }

  let { entry, onclose, onopenwith }: Props = $props();
</script>

<BottomSheet title={t('props.title')} {onclose}>
  <div class="head">
    <span class="icon"><Icon name={categoryIcon(entry)} size={26} stroke={1.6} /></span>
    <div class="titles">
      <h3>{entry.name}</h3>
      <p>{t(categoryLabelKey(entry))}{entry.readOnly ? ` · ${t('props.readOnly')}` : ''}</p>
    </div>
  </div>
  <dl class="rows">
    <div>
      <dt>{t('props.size')}</dt>
      <dd>{entry.isDir ? '—' : formatSize(entry.size)}</dd>
    </div>
    <div>
      <dt>{t('props.modified')}</dt>
      <dd>{formatDate(entry.modified, localeTag())}</dd>
    </div>
    <div>
      <dt>{t('props.type')}</dt>
      <dd>{entry.mime}</dd>
    </div>
    <div class="path">
      <dt>{t('props.path')}</dt>
      <dd>{entry.path}</dd>
    </div>
  </dl>
  <div class="actions">
    <button class="btn btn-secondary btn-block" onclick={onopenwith}>
      <Icon name="external-link" size={18} />
      {t('common.openWith')}
    </button>
  </div>
</BottomSheet>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-5) var(--space-4);
  }
  .icon {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .titles h3 {
    font-size: 1.0625rem;
    word-break: break-word;
  }
  .titles p {
    color: var(--ink-3);
    font-size: 0.8125rem;
    margin-top: 2px;
  }
  .rows {
    margin: 0;
    padding: 0 var(--space-5);
    display: grid;
    gap: var(--space-3);
  }
  .rows div {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: var(--space-3);
    align-items: baseline;
  }
  dt {
    color: var(--ink-3);
    font-size: 0.8125rem;
  }
  dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--ink);
    word-break: break-word;
  }
  .path dd {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    line-height: 1.5;
    color: var(--ink-2);
    background: var(--paper-sunken);
    padding: var(--space-2);
    border-radius: var(--radius-sm);
  }
  .actions {
    padding: var(--space-4) var(--space-5) var(--space-3);
  }
</style>
