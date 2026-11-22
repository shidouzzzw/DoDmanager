<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import type { JobProgress } from '../types';
  import { formatSize } from '../services/format';
  import Icon from './Icon.svelte';

  interface Props {
    job: JobProgress;
    label: string;
    oncancel: () => void;
    offset?: number;
  }

  let { job, label, oncancel, offset = 0 }: Props = $props();

  const ratio = $derived(job.total > 0 ? Math.min(1, job.done / job.total) : 0);
</script>

<div class="job" style="--job-offset: {offset}px" role="status" aria-live="polite">
  <div class="row">
    <span class="text">
      <strong>{label}</strong>
      <small>{job.current || t('job.preparing')}</small>
    </span>
    <span class="numbers">
      {#if job.total > 0}
        {formatSize(job.done)} / {formatSize(job.total)}
      {:else}
        {formatSize(job.done)}
      {/if}
    </span>
    <button class="cancel" onclick={oncancel} aria-label={t('job.cancel')}>
      <Icon name="x" size={18} />
    </button>
  </div>
  <div class="bar" aria-hidden="true">
    <div class="fill" class:indeterminate={job.total === 0} style="width: {ratio * 100}%"></div>
  </div>
</div>

<style>
  .job {
    position: fixed;
    left: 0;
    right: 0;
    bottom: calc(var(--job-offset) + var(--safe-bottom));
    z-index: 34;
    background: var(--paper-raised);
    border-top: 1px solid var(--line);
    box-shadow: var(--shadow-md);
    padding: var(--space-3) var(--space-4);
    animation: rise var(--dur-base) var(--ease-spring);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .text strong {
    font-size: 0.875rem;
    font-weight: 620;
  }
  .text small {
    color: var(--ink-3);
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .numbers {
    font-variant-numeric: tabular-nums;
    font-size: 0.75rem;
    color: var(--ink-3);
  }
  .cancel {
    width: 40px;
    height: 40px;
    min-width: 40px;
    min-height: 40px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--paper-sunken);
  }
  .bar {
    margin-top: var(--space-2);
    height: 6px;
    border-radius: 999px;
    background: var(--paper-sunken);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
    transition: width 120ms linear;
  }
  .fill.indeterminate {
    width: 35% !important;
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(320%);
    }
  }
</style>
