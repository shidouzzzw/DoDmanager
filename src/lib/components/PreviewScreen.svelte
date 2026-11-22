<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { t } from '../i18n';
  import { assetUrl, openWith, previewAsset, readFile } from '../services/files';
  import { openExternal } from '../services/settings';
  import { middleEllipsis } from '../services/format';
  import { app } from '../state/app.svelte';
  import type { ReadFileResult } from '../types';
  import { inlinePreview } from '../ui/fileMeta';
  import { highlightCode, renderMarkdown } from '../ui/highlight';
  import EmptyState from './EmptyState.svelte';
  import Icon from './Icon.svelte';

  interface PdfPage {
    getViewport(params: { scale: number }): { width: number; height: number };
    render(params: Record<string, unknown>): { promise: Promise<void> };
  }
  interface PdfDocument {
    numPages: number;
    getPage(page: number): Promise<PdfPage>;
  }

  const entry = $derived(app.previewItems[app.previewIndex] ?? null);
  const kind = $derived(entry ? inlinePreview(entry) : 'none');
  const total = $derived(app.previewItems.length);

  let text = $state<ReadFileResult | null>(null);
  let loading = $state(true);
  let failure = $state<string | null>(null);
  let source = $state('');
  let pdfCanvas = $state<HTMLCanvasElement | null>(null);
  let pdfPage = $state(1);
  let pdfPages = $state(0);
  let pdfLoading = $state(false);
  let token = 0;
  let pdfDocument: PdfDocument | null = null;

  let scale = $state(1);
  let offsetX = $state(0);
  let offsetY = $state(0);
  // Jest girdisi tepkisel değil; SvelteMap gereksiz yeniden çizim üretir.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const pointers = new Map<number, { x: number; y: number }>();
  let pinchStart = 0;
  let pinchScale = 1;
  let swipeStart = 0;
  let swipeDelta = 0;
  let lastTap = 0;

  function resetGestures() {
    scale = 1;
    offsetX = 0;
    offsetY = 0;
    pointers.clear();
    swipeDelta = 0;
  }

  $effect(() => {
    const current = entry;
    const previewKind = kind;
    if (!current) return;
    const job = ++token;
    loading = true;
    failure = null;
    text = null;
    source = '';
    pdfDocument = null;
    pdfPages = 0;
    pdfPage = 1;
    resetGestures();

    if (previewKind === 'code' || previewKind === 'markdown' || previewKind === 'text') {
      readFile(current.path)
        .then((result) => {
          if (job !== token) return;
          text = result;
          loading = false;
        })
        .catch((error) => {
          if (job !== token) return;
          failure = app.fail(error).localized();
          loading = false;
        });
      return;
    }

    previewAsset(current.path)
      .then(async (result) => {
        if (job !== token) return;
        source = assetUrl(result.path);
        loading = false;
        if (previewKind === 'pdf') await renderPdf(result.path, job);
      })
      .catch((error) => {
        if (job !== token) return;
        failure = app.fail(error).localized();
        loading = false;
      });
  });

  onDestroy(() => {
    token += 1;
    pdfDocument = null;
  });

  async function renderPdf(path: string, job: number) {
    pdfLoading = true;
    try {
      const pdfjs = await import('pdfjs-dist');
      const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
      pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
      const document = (await pdfjs.getDocument({ url: assetUrl(path) })
        .promise) as unknown as PdfDocument;
      if (job !== token) return;
      pdfDocument = document;
      pdfPages = document.numPages;
      await drawPage(1, job);
    } catch (error) {
      if (job === token) failure = t('errors.UNREADABLE');
      void error;
    } finally {
      if (job === token) pdfLoading = false;
    }
  }

  async function drawPage(number: number, job = token) {
    if (!pdfDocument || !pdfCanvas) return;
    const page = await pdfDocument.getPage(number);
    if (job !== token) return;
    const base = page.getViewport({ scale: 1 });
    const width = pdfCanvas.parentElement?.clientWidth ?? 360;
    const viewportScale = Math.min(3, (width - 24) / base.width);
    const viewport = page.getViewport({ scale: viewportScale });
    pdfCanvas.width = Math.floor(viewport.width);
    pdfCanvas.height = Math.floor(viewport.height);
    const context = pdfCanvas.getContext('2d');
    if (!context) return;
    await page.render({ canvasContext: context, viewport, canvas: pdfCanvas }).promise;
    pdfPage = number;
  }

  async function changePage(delta: number) {
    if (!pdfDocument) return;
    const next = Math.min(pdfPages, Math.max(1, pdfPage + delta));
    if (next === pdfPage) return;
    pdfLoading = true;
    await drawPage(next);
    pdfLoading = false;
  }

  function nextFile() {
    if (app.previewIndex < total - 1) {
      app.previewIndex += 1;
    }
  }

  function previousFile() {
    if (app.previewIndex > 0) {
      app.previewIndex -= 1;
    }
  }

  function pointerDown(event: PointerEvent) {
    if (kind !== 'image') return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
    if (pointers.size === 1) swipeStart = event.clientX;
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      pinchStart = Math.hypot(a.x - b.x, a.y - b.y);
      pinchScale = scale;
    }
  }

  function pointerMove(event: PointerEvent) {
    if (!pointers.has(event.pointerId)) return;
    const previous = pointers.get(event.pointerId);
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.size >= 2) {
      const [a, b] = [...pointers.values()];
      const distance = Math.hypot(a.x - b.x, a.y - b.y);
      if (pinchStart > 0) {
        scale = Math.min(6, Math.max(1, pinchScale * (distance / pinchStart)));
        if (scale === 1) {
          offsetX = 0;
          offsetY = 0;
        }
      }
      return;
    }
    if (scale > 1 && previous) {
      offsetX += event.clientX - previous.x;
      offsetY += event.clientY - previous.y;
    } else {
      swipeDelta = event.clientX - swipeStart;
    }
  }

  function pointerUp(event: PointerEvent) {
    pointers.delete(event.pointerId);
    if (pointers.size > 0) return;
    const now = Date.now();
    if (Math.abs(swipeDelta) < 6 && now - lastTap < 280) {
      // Çift dokunuş: yakınlaştır / sığdır.
      scale = scale > 1.2 ? 1 : 2.4;
      offsetX = 0;
      offsetY = 0;
      lastTap = 0;
      swipeDelta = 0;
      pinchStart = 0;
      return;
    }
    lastTap = now;
    if (scale <= 1.05 && Math.abs(swipeDelta) > 64) {
      if (swipeDelta < 0) nextFile();
      else previousFile();
    }
    if (scale < 1) scale = 1;
    if (scale <= 1.02) {
      offsetX = 0;
      offsetY = 0;
    }
    swipeDelta = 0;
    pinchStart = 0;
  }

  async function openElsewhere() {
    if (!entry) return;
    try {
      await openWith(entry.path);
    } catch (error) {
      app.fail(error);
    }
  }

  function interceptLinks(event: MouseEvent) {
    const anchor = (event.target as HTMLElement).closest('a');
    if (!anchor) return;
    event.preventDefault();
    const href = anchor.getAttribute('href');
    if (!href) return;
    void openExternal(href).catch(() => {});
  }

  const rendered = $derived(
    kind === 'markdown'
      ? renderMarkdown(text?.text ?? '')
      : highlightCode(text?.text ?? '', entry?.name.split('.').pop() ?? ''),
  );
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') app.closePreview();
    if (event.key === 'ArrowRight') nextFile();
    if (event.key === 'ArrowLeft') previousFile();
  }}
/>

<div class="screen preview">
  <header class="appbar pad-x">
    <button class="icon-btn" aria-label={t('a11y.back')} onclick={() => app.closePreview()}>
      <Icon name="chevron-left" size={24} />
    </button>
    <div class="titles">
      <h1>{middleEllipsis(entry?.name ?? '', 26)}</h1>
      {#if total > 1}
        <span>{t('preview.counter', { index: app.previewIndex + 1, total })}</span>
      {:else if entry}
        <span>{entry.mime}</span>
      {/if}
    </div>
    <button class="icon-btn" aria-label={t('common.openWith')} onclick={openElsewhere}>
      <Icon name="external-link" size={20} />
    </button>
  </header>

  <div
    class="stage"
    class:zoomed={scale > 1}
    onpointerdown={pointerDown}
    onpointermove={pointerMove}
    onpointerup={pointerUp}
    onpointercancel={pointerUp}
  >
    {#if failure}
      <EmptyState icon="circle-alert" title={failure} hint={entry?.path}>
        {#snippet action()}
          <button class="btn btn-primary" onclick={openElsewhere}>{t('preview.openWith')}</button>
        {/snippet}
      </EmptyState>
    {:else if loading}
      <div class="loader">
        <span class="spinner" aria-hidden="true"></span>
        <p>{t('common.loading')}</p>
      </div>
    {:else if kind === 'image' && source}
      <img
        class="image"
        src={source}
        alt={entry?.name ?? ''}
        style="transform: translate3d({offsetX}px, {offsetY}px, 0) scale({scale})"
        draggable="false"
      />
      {#if total > 1}<p class="hint">{t('preview.swipeHint')}</p>{/if}
    {:else if kind === 'video'}
      <video
        class="media"
        src={source}
        controls
        playsinline
        preload="metadata"
        aria-label={entry?.name}
      ></video>
    {:else if kind === 'audio'}
      <div class="audio card">
        <div class="audio-art"><Icon name="music" size={44} stroke={1.3} /></div>
        <h2>{middleEllipsis(entry?.name ?? '', 30)}</h2>
        <audio src={source} controls preload="metadata" aria-label={entry?.name}></audio>
      </div>
    {:else if kind === 'pdf'}
      <div class="pdf">
        {#if pdfLoading && !pdfPages}
          <div class="loader">
            <span class="spinner"></span>
            <p>{t('preview.pdfLoading')}</p>
          </div>
        {/if}
        <canvas bind:this={pdfCanvas} class="pdf-canvas" aria-label={entry?.name}></canvas>
        {#if pdfPages > 1}
          <div class="pdf-controls">
            <button
              class="btn btn-secondary"
              onclick={() => changePage(-1)}
              disabled={pdfPage <= 1 || pdfLoading}
              aria-label={t('preview.previous')}
            >
              <Icon name="chevron-left" size={18} />
            </button>
            <span>{t('preview.counter', { index: pdfPage, total: pdfPages })}</span>
            <button
              class="btn btn-secondary"
              onclick={() => changePage(1)}
              disabled={pdfPage >= pdfPages || pdfLoading}
              aria-label={t('preview.next')}
            >
              <Icon name="chevron-right" size={18} />
            </button>
          </div>
        {/if}
      </div>
    {:else if kind === 'markdown'}
      <article class="prose" onclick={interceptLinks}>
        {#if text?.truncated}<p class="truncated">{t('preview.textTruncated')}</p>{/if}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -- renderMarkdown HTML kaçışı yapar -->
        {@html rendered}
      </article>
    {:else if kind === 'code' || kind === 'text'}
      <div class="code-wrap">
        {#if text?.truncated}<p class="truncated">{t('preview.textTruncated')}</p>{/if}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -- highlightCode HTML kaçışı yapar -->
        <pre class="code"><code>{@html rendered}</code></pre>
      </div>
    {:else}
      <EmptyState
        icon="file-question"
        title={t('preview.unsupported')}
        hint={t('preview.unsupportedHint')}
      >
        {#snippet action()}
          <button class="btn btn-primary" onclick={openElsewhere}>{t('preview.openWith')}</button>
        {/snippet}
      </EmptyState>
    {/if}
  </div>

  {#if total > 1 && (kind === 'image' || kind === 'video')}
    <div class="pager">
      <button
        class="btn btn-secondary"
        onclick={previousFile}
        disabled={app.previewIndex === 0}
        aria-label={t('preview.previous')}
      >
        <Icon name="chevron-left" size={18} />
      </button>
      <span class="dots" aria-hidden="true">
        {#each app.previewItems as _, index (index)}
          <span class="dot" class:on={index === app.previewIndex}></span>
        {/each}
      </span>
      <button
        class="btn btn-secondary"
        onclick={nextFile}
        disabled={app.previewIndex >= total - 1}
        aria-label={t('preview.next')}
      >
        <Icon name="chevron-right" size={18} />
      </button>
    </div>
  {/if}
</div>

<style>
  .preview {
    background: var(--paper);
  }
  .appbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
    border-bottom: 1px solid var(--line);
    z-index: 5;
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  .titles h1 {
    font-family: var(--font-display);
    font-size: 1rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .titles span {
    font-size: 0.6875rem;
    color: var(--ink-3);
  }
  .stage {
    flex: 1;
    position: relative;
    display: grid;
    place-items: center;
    overflow: hidden;
    padding: var(--space-3);
    touch-action: none;
  }
  .stage.zoomed {
    cursor: grab;
  }
  .image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--radius-md);
    transition: transform 80ms linear;
    box-shadow: var(--shadow-md);
    user-select: none;
  }
  .media {
    width: 100%;
    max-height: 100%;
    border-radius: var(--radius-md);
    background: #000;
  }
  .hint {
    position: absolute;
    bottom: var(--space-4);
    font-size: 0.6875rem;
    color: var(--ink-3);
    background: var(--paper-raised);
    padding: 6px 12px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--line);
  }
  .loader {
    display: grid;
    place-items: center;
    gap: var(--space-3);
    color: var(--ink-3);
    font-size: 0.8125rem;
  }
  .spinner {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 3px solid var(--line-strong);
    border-top-color: var(--accent);
    animation: spin 800ms linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .audio {
    display: grid;
    gap: var(--space-4);
    justify-items: center;
    padding: var(--space-5);
    width: min(420px, 100%);
  }
  .audio-art {
    width: 120px;
    height: 120px;
    border-radius: 32px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .audio h2 {
    font-size: 1rem;
    text-align: center;
    word-break: break-word;
  }
  .audio audio {
    width: 100%;
  }
  .pdf {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    overflow: auto;
  }
  .pdf-canvas {
    max-width: 100%;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
    background: #fff;
  }
  .pdf-controls {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: 0.8125rem;
    color: var(--ink-3);
  }
  .prose,
  .code-wrap {
    width: 100%;
    height: 100%;
    overflow: auto;
    padding: var(--space-4);
    background: var(--paper-raised);
    border-radius: var(--radius-md);
    border: 1px solid var(--line);
    user-select: text;
  }
  .prose {
    font-size: 0.9375rem;
    line-height: 1.65;
    color: var(--ink-2);
  }
  .truncated {
    font-size: 0.75rem;
    color: var(--ink-3);
    background: var(--paper-sunken);
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-3);
  }
  .code {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    line-height: 1.6;
    white-space: pre;
    color: var(--ink);
  }
  .pager {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4) calc(var(--safe-bottom) + var(--space-2));
    border-top: 1px solid var(--line);
  }
  .dots {
    display: flex;
    gap: 5px;
    overflow: hidden;
    max-width: 50vw;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--line-strong);
    flex: none;
  }
  .dot.on {
    background: var(--accent);
    width: 16px;
    border-radius: 3px;
  }

  .prose :global(h1),
  .prose :global(h2),
  .prose :global(h3) {
    font-family: var(--font-display);
    margin: var(--space-4) 0 var(--space-2);
    color: var(--ink);
  }
  .prose :global(p) {
    margin: var(--space-2) 0;
  }
  .prose :global(ul) {
    list-style: disc;
    padding-left: var(--space-5);
    margin: var(--space-2) 0;
  }
  .prose :global(a) {
    color: var(--accent);
  }
  .prose :global(code) {
    font-family: var(--font-mono);
    font-size: 0.8125em;
    background: var(--paper-sunken);
    padding: 1px 5px;
    border-radius: 4px;
  }
  .prose :global(pre) {
    background: var(--paper-sunken);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    overflow-x: auto;
  }
  .prose :global(pre code) {
    background: none;
    padding: 0;
  }
  .prose :global(blockquote) {
    border-left: 3px solid var(--accent);
    padding-left: var(--space-3);
    color: var(--ink-3);
    margin: var(--space-2) 0;
  }
  .prose :global(hr) {
    border: 0;
    border-top: 1px solid var(--line);
    margin: var(--space-4) 0;
  }
  .code :global(.tok-comment) {
    color: var(--ink-3);
    font-style: italic;
  }
  .code :global(.tok-string) {
    color: var(--cat-audio);
  }
  .code :global(.tok-keyword) {
    color: var(--accent);
    font-weight: 600;
  }
  .code :global(.tok-number) {
    color: var(--cat-document);
  }
</style>
