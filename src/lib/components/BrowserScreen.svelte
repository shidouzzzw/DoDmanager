<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import { t } from '../i18n';
  import { haptic } from '../services/haptics';
  import {
    categoryFilter,
    deleteEntries,
    newJobId,
    openWith as openWithFile,
    renameEntry,
    shareFiles,
    zipCreate,
    zipExtract,
    createDir,
    createFile,
    moveEntries,
  } from '../services/files';
  import { recentAdd, trashRestore } from '../services/settings';
  import { SearchSession } from '../services/search';
  import { app } from '../state/app.svelte';
  import type { FileEntry } from '../types';
  import {
    inlinePreview,
    isZip,
    prepareEntries,
    previewableFiles,
    rangeIndexes,
    suggestName,
    type FilterKey,
  } from '../ui/fileMeta';
  import ActionSheet, { type SheetAction } from './ActionSheet.svelte';
  import BreadcrumbBar from './BreadcrumbBar.svelte';
  import EmptyState from './EmptyState.svelte';
  import FileTile from './FileTile.svelte';
  import Icon from './Icon.svelte';
  import PromptSheet from './PromptSheet.svelte';
  import PropertiesSheet from './PropertiesSheet.svelte';
  import SelectionBar from './SelectionBar.svelte';
  import SortFilterSheet from './SortFilterSheet.svelte';
  import type { Crumb } from '../types';

  type SheetKind =
    'actions' | 'sort' | 'rename' | 'newfolder' | 'newfile' | 'zip' | 'props' | 'fab';

  let scrollEl = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewportH = $state(600);
  let viewportW = $state(360);
  let scrolled = $state(false);
  let sheet = $state<SheetKind | null>(null);
  let target = $state<FileEntry | null>(null);
  let busy = $state(false);
  let searchOpen = $state(false);
  let searchInput = $state<HTMLInputElement | null>(null);

  let rangeActive = false;
  let drag: { x: number; y: number; active: boolean } | null = null;
  let dropTarget = $state<string | null>(null);
  let session: SearchSession | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let observer: ResizeObserver | null = null;

  const pad = 16;
  const gap = 12;

  const sourceEntries = $derived(
    app.query.trim() ? app.searchHits.map((hit) => hit.entry) : (app.listing?.entries ?? []),
  );

  const items = $derived(
    prepareEntries(sourceEntries, {
      sort: app.settings.sort,
      desc: app.settings.sortDesc,
      filter: app.filter as FilterKey,
    }),
  );

  const columns = $derived(app.settings.view === 'grid' ? app.settings.gridColumns : 1);
  const cellW = $derived(
    Math.max(72, Math.floor((viewportW - pad * 2 - gap * (columns - 1)) / columns)),
  );
  const tileH = $derived(app.settings.view === 'grid' ? cellW + 46 : 68);
  const rowH = $derived(tileH + (app.settings.view === 'grid' ? gap : 0));
  const rows = $derived(Math.max(1, Math.ceil(items.length / columns)));
  const totalH = $derived(pad * 2 + rows * rowH);
  const startRow = $derived(Math.max(0, Math.floor((scrollTop - pad) / rowH) - 2));
  const endRow = $derived(Math.min(rows - 1, Math.ceil((scrollTop + viewportH - pad) / rowH) + 1));
  const windowItems = $derived(
    items
      .slice(startRow * columns, (endRow + 1) * columns)
      .map((entry, offset) => ({ entry, index: startRow * columns + offset })),
  );

  const selecting = $derived(app.selected.size > 0);
  const selectedEntries = $derived(items.filter((entry) => app.selected.has(entry.path)));
  const rootLabel = $derived(app.navStack[0]?.name ?? app.currentLocation?.label ?? '');
  const locationLabel = $derived(app.currentLocation?.label ?? rootLabel);

  function position(index: number): string {
    const row = Math.floor(index / columns);
    const col = index % columns;
    return `translate3d(${pad + col * (cellW + gap)}px, ${pad + row * rowH}px, 0)`;
  }

  function onScroll() {
    scrollTop = scrollEl?.scrollTop ?? 0;
    scrolled = scrollTop > 8;
    if (rangeActive && app.anchorIndex >= 0) {
      // Kaydırırken aralık seçimi genişlemesin; sadece dokunma ile genişler.
    }
  }

  onMount(() => {
    const measure = () => {
      if (!scrollEl) return;
      viewportH = scrollEl.clientHeight;
      viewportW = scrollEl.clientWidth;
    };
    measure();
    observer = new ResizeObserver(measure);
    if (scrollEl) observer.observe(scrollEl);
    window.addEventListener('pointerup', endPointer);
    return () => {
      observer?.disconnect();
      window.removeEventListener('pointerup', endPointer);
    };
  });

  onDestroy(() => {
    if (searchTimer) clearTimeout(searchTimer);
    void session?.stop();
  });

  function endPointer() {
    rangeActive = false;
    drag = null;
    dropTarget = null;
  }

  async function onActivate(entry: FileEntry, index: number) {
    if (selecting) {
      app.toggleSelection(entry.path);
      haptic('select');
      return;
    }
    if (entry.isDir) {
      await app.openFolder(entry);
      if (scrollEl) scrollEl.scrollTop = 0;
      return;
    }
    void recentAdd(entry.path, app.currentLocation?.id)
      .then(() => app.refreshRecents())
      .catch(() => {});
    const kind = inlinePreview(entry);
    if (kind !== 'none') {
      const files = previewableFiles(items);
      const startIndex = Math.max(
        0,
        files.findIndex((item) => item.path === entry.path),
      );
      app.openPreview(files.length ? files : [entry], startIndex);
    } else {
      try {
        await openWithFile(entry.path);
      } catch (error) {
        app.fail(error);
      }
    }
    void index;
  }

  function onLongPress(entry: FileEntry, index: number) {
    haptic('select');
    if (app.anchorIndex < 0) app.anchorIndex = index;
    rangeActive = true;
    if (app.isSelected(entry.path) && app.selected.size > 1) return;
    app.toggleSelection(entry.path);
  }

  function onHover(index: number) {
    if (!rangeActive || app.anchorIndex < 0) return;
    const next = new SvelteSet(app.selected);
    for (const position of rangeIndexes(app.anchorIndex, index)) {
      const entry = items[position];
      if (entry) next.add(entry.path);
    }
    app.selected = next;
  }

  function onPointerDownItem(event: PointerEvent, entry: FileEntry, index: number) {
    if (!selecting || !app.isSelected(entry.path)) return;
    drag = { x: event.clientX, y: event.clientY, active: false };
    void index;
  }

  function onPointerMoveItem(event: PointerEvent) {
    if (!drag) return;
    if (!drag.active && Math.hypot(event.clientX - drag.x, event.clientY - drag.y) > 14) {
      drag.active = true;
      rangeActive = false;
      haptic('drag');
    }
    if (!drag.active) return;
    const element = document.elementFromPoint(event.clientX, event.clientY);
    const holder = element?.closest('[data-dir-path]') as HTMLElement | null;
    const path = holder?.getAttribute('data-dir-path') ?? null;
    dropTarget = path && !app.selected.has(path) ? path : null;
  }

  async function onPointerUpItem() {
    const destination = dropTarget;
    const wasDragging = drag?.active ?? false;
    drag = null;
    dropTarget = null;
    if (wasDragging && destination) {
      await moveSelectionTo(destination);
    }
  }

  function navigateCrumb(_crumb: Crumb, index: number) {
    if (index === app.navStack.length - 1) return;
    app.navStack = app.navStack.slice(0, index + 1);
    void app.loadPath(app.currentPath);
    if (scrollEl) scrollEl.scrollTop = 0;
  }

  function openTarget(kind: 'rename' | 'props' | 'zip') {
    sheet = kind;
  }

  async function doRename(newName: string) {
    if (!target) return;
    busy = true;
    const previous = target.name;
    const path = target.path;
    try {
      const updated = await renameEntry(path, newName);
      sheet = null;
      target = null;
      await app.reload();
      app.toast(t('snackbar.renamed'), async () => {
        await renameEntry(updated.path, previous);
        await app.reload();
        app.toast(t('snackbar.undone'));
      });
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function doCreate(kind: 'folder' | 'file', name: string) {
    busy = true;
    try {
      if (kind === 'folder') await createDir(app.currentPath, name);
      else await createFile(app.currentPath, name);
      sheet = null;
      await app.reload();
      app.toast(t('snackbar.created'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function doZip(name: string) {
    const sources = selectedEntries.length
      ? selectedEntries.map((entry) => entry.path)
      : target
        ? [target.path]
        : [];
    if (!sources.length) return;
    busy = true;
    try {
      await zipCreate(newJobId(), sources, app.currentPath, name);
      sheet = null;
      await app.reload();
      app.toast(t('snackbar.zipped'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function doUnzip(entry: FileEntry) {
    sheet = null;
    busy = true;
    try {
      const base = entry.name.replace(/\.zip$/i, '');
      const folderName = suggestName(items, base);
      const folder = await createDir(app.currentPath, folderName);
      await zipExtract(newJobId(), entry.path, folder.path);
      await app.reload();
      app.toast(t('snackbar.unzipped'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function deleteSelection() {
    if (!selectedEntries.length) return;
    const entries = selectedEntries;
    app.clearSelection();
    haptic('delete');
    try {
      const trashed = await deleteEntries(
        newJobId(),
        entries.map((entry) => ({ path: entry.path, parent: app.currentPath })),
        locationLabel,
      );
      await app.reload();
      app.toast(t('snackbar.deleted', { count: trashed.length }), async () => {
        for (const item of trashed) await trashRestore(item.id);
        await app.reload();
        app.toast(t('snackbar.undone'));
      });
    } catch (error) {
      app.fail(error);
    }
  }

  async function shareSelection() {
    const paths = selectedEntries.map((entry) => entry.path);
    if (!paths.length) return;
    try {
      await shareFiles(paths);
    } catch (error) {
      app.fail(error);
    }
  }

  async function moveSelectionTo(destination: string) {
    const paths = selectedEntries.map((entry) => entry.path);
    if (!paths.length) return;
    const source = app.currentPath;
    app.clearSelection();
    try {
      await moveEntries(newJobId(), paths, destination);
      await app.reload();
      app.toast(t('snackbar.moved', { count: paths.length }), async () => {
        await moveEntries(
          newJobId(),
          paths.map((path) => path),
          source,
        ).catch(() => {});
        await app.reload();
      });
    } catch (error) {
      app.fail(error);
    }
  }

  function beginTransfer(mode: 'copy' | 'move', entries: FileEntry[]) {
    const paths = entries.map((entry) => entry.path);
    if (!paths.length) return;
    app.clearSelection();
    sheet = null;
    app.beginTransfer(mode, paths);
  }

  const actions = $derived.by<SheetAction[]>(() => {
    const selection = selectedEntries;
    const single = selection.length === 1 ? selection[0] : target;
    const list: SheetAction[] = [];
    if (selection.length <= 1 && single) {
      if (single.isDir) list.push({ id: 'open', label: t('actions.open'), icon: 'folder-open' });
      else list.push({ id: 'open', label: t('actions.open'), icon: 'file' });
      if (!single.isDir)
        list.push({ id: 'openwith', label: t('actions.openWith'), icon: 'external-link' });
    }
    list.push({ id: 'share', label: t('actions.share'), icon: 'share-2' });
    if (selection.length === 1 || (selection.length === 0 && single)) {
      list.push({ id: 'rename', label: t('actions.rename'), icon: 'pencil' });
    }
    list.push({ id: 'copy', label: t('actions.copy'), icon: 'copy' });
    list.push({ id: 'move', label: t('actions.move'), icon: 'folder-input' });
    if (selection.some((entry) => !entry.isDir)) {
      list.push({ id: 'zip', label: t('actions.zipCreate'), icon: 'archive' });
    }
    if (single && isZip(single) && selection.length <= 1) {
      list.push({ id: 'unzip', label: t('actions.zipExtract'), icon: 'package' });
    }
    if (selection.length === 1 || (selection.length === 0 && single)) {
      list.push({ id: 'props', label: t('actions.properties'), icon: 'info' });
    }
    if (selection.length > 1) {
      list.push({ id: 'selectall', label: t('actions.selectAll'), icon: 'square-check-big' });
    }
    list.push({ id: 'delete', label: t('actions.delete'), icon: 'trash-2', danger: true });
    return list;
  });

  async function handleAction(id: string) {
    const selection = selectedEntries.length ? selectedEntries : target ? [target] : [];
    const single = selection[0];
    switch (id) {
      case 'open':
        sheet = null;
        if (single)
          await onActivate(
            single,
            items.findIndex((e) => e.path === single.path),
          );
        break;
      case 'openwith':
        sheet = null;
        if (single)
          await openWithFile(single.path).catch((error) => {
            app.fail(error);
          });
        break;
      case 'share':
        sheet = null;
        await shareSelection();
        break;
      case 'rename':
        if (single) openTarget('rename');
        break;
      case 'copy':
        beginTransfer('copy', selection);
        break;
      case 'move':
        beginTransfer('move', selection);
        break;
      case 'zip':
        openTarget('zip');
        break;
      case 'unzip':
        if (single) await doUnzip(single);
        break;
      case 'props':
        openTarget('props');
        break;
      case 'selectall':
        sheet = null;
        app.selectAll(items);
        break;
      case 'delete':
        sheet = null;
        await deleteSelection();
        break;
    }
  }

  function onSearchInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    app.query = value;
    app.searchHits = [];
    app.searchScanned = 0;
    if (searchTimer) clearTimeout(searchTimer);
    if (!value.trim()) {
      void session?.stop();
      session = null;
      app.searching = false;
      return;
    }
    app.searching = true;
    searchTimer = setTimeout(() => {
      void session?.stop();
      session = new SearchSession({
        onHits: (hits) => {
          app.searchHits = [...app.searchHits, ...hits];
        },
        onDone: (done) => {
          app.searching = false;
          app.searchScanned = done.scanned;
        },
        onError: (error) => {
          app.searching = false;
          app.fail(error);
        },
      });
      void session.start(app.currentPath, value, categoryFilter(app.filter));
    }, 260);
  }

  function changeFilter(filter: FilterKey) {
    app.filter = filter;
    if (app.query.trim() && session) {
      app.searchHits = [];
      app.searching = true;
      void session.start(app.currentPath, app.query, categoryFilter(filter));
    }
  }

  function toggleSearch() {
    searchOpen = !searchOpen;
    if (!searchOpen) {
      app.query = '';
      app.searchHits = [];
      app.searching = false;
      void session?.stop();
      session = null;
    } else {
      requestAnimationFrame(() => searchInput?.focus());
    }
  }

  function goUp() {
    void app.back();
  }

  const emptyTitle = $derived(
    app.query.trim() ? t('browser.emptySearch') : t('browser.emptyFolder'),
  );
  const emptyHint = $derived(
    app.query.trim() ? t('browser.emptySearchHint') : t('browser.emptyFolderHint'),
  );
  const filters: { key: FilterKey; label: string }[] = [
    { key: 'all', label: t('browser.filterAll') },
    { key: 'image', label: t('browser.filterImages') },
    { key: 'video', label: t('browser.filterVideos') },
    { key: 'document', label: t('browser.filterDocuments') },
    { key: 'audio', label: t('browser.filterAudio') },
    { key: 'archive', label: t('browser.filterArchives') },
  ];

  const zipInitial = $derived(
    `${(selectedEntries.length === 1 ? selectedEntries[0].name : app.currentName || 'arsiv').replace(/\.[^.]+$/, '')}.zip`,
  );
</script>

<div class="screen">
  <header class="appbar pad-x" class:raised={scrolled}>
    <button class="icon-btn" aria-label={t('a11y.back')} onclick={goUp}>
      <Icon name="chevron-left" size={24} />
    </button>
    <div class="titles">
      <h1>{app.query.trim() ? t('browser.searchIn') : app.currentName || rootLabel}</h1>
      <span>
        {#if app.query.trim()}
          {#if app.searching}
            {t('browser.searching')}
          {:else}
            {t('browser.searchResults', { count: items.length })}
          {/if}
        {:else}
          {t('browser.itemsCount', { count: items.length })}
        {/if}
      </span>
    </div>
    <button
      class="icon-btn"
      aria-label={t('a11y.search')}
      aria-pressed={searchOpen}
      onclick={toggleSearch}
    >
      <Icon name={searchOpen ? 'x' : 'search'} size={21} />
    </button>
    <button
      class="icon-btn"
      aria-label={t('a11y.viewToggle')}
      onclick={() => app.updateSettings({ view: app.settings.view === 'grid' ? 'list' : 'grid' })}
    >
      <Icon name={app.settings.view === 'grid' ? 'list' : 'layout-grid'} size={21} />
    </button>
    <button class="icon-btn" aria-label={t('a11y.more')} onclick={() => (sheet = 'fab')}>
      <Icon name="more-vertical" size={21} />
    </button>
  </header>

  <div class="searchbar pad-x" class:open={searchOpen}>
    {#if searchOpen}
      <div class="search-field">
        <Icon name="search" size={18} />
        <input
          bind:this={searchInput}
          value={app.query}
          oninput={onSearchInput}
          placeholder={t('browser.searchPlaceholder')}
          aria-label={t('a11y.search')}
          autocomplete="off"
          autocapitalize="none"
          spellcheck="false"
          enterkeyhint="search"
        />
        {#if app.query}
          <button
            class="icon-btn small"
            aria-label={t('a11y.clearSearch')}
            onclick={() => {
              app.query = '';
              app.searchHits = [];
              app.searching = false;
              void session?.stop();
              session = null;
              searchInput?.focus();
            }}
          >
            <Icon name="x" size={16} />
          </button>
        {/if}
      </div>
    {:else}
      <BreadcrumbBar crumbs={app.navStack} onnavigate={navigateCrumb} />
    {/if}
  </div>

  <div class="tools">
    <div class="chips">
      {#each filters as item (item.key)}
        <button
          class="chip"
          class:is-active={app.filter === item.key}
          aria-pressed={app.filter === item.key}
          onclick={() => changeFilter(item.key)}
        >
          {item.label}
        </button>
      {/each}
    </div>
    <button class="chip sort" onclick={() => (sheet = 'sort')}>
      <Icon name="arrow-up-down" size={15} />
      {app.settings.sort === 'name'
        ? t('browser.sortName')
        : app.settings.sort === 'date'
          ? t('browser.sortDate')
          : app.settings.sort === 'size'
            ? t('browser.sortSize')
            : t('browser.sortType')}
    </button>
  </div>

  <div
    class="content"
    class:selection={selecting}
    bind:this={scrollEl}
    onscroll={onScroll}
    role="list"
    aria-label={app.currentName}
  >
    {#if app.loading && !app.listing}
      <div class="skeletons" style="--cols: {app.settings.gridColumns}">
        {#each Array(app.settings.gridColumns * 4) as _, index (index)}
          <div class="sk">
            <div class="skeleton" style="aspect-ratio: 1/1"></div>
            <div class="skeleton line"></div>
          </div>
        {/each}
      </div>
    {:else if app.loadingError}
      <EmptyState icon="circle-alert" title={app.loadingError.localized()} hint={app.currentPath}>
        {#snippet action()}
          <button class="btn btn-primary" onclick={() => app.reload()}>{t('common.retry')}</button>
        {/snippet}
      </EmptyState>
    {:else if items.length === 0}
      <EmptyState
        icon={app.query.trim() ? 'search' : 'folder-open'}
        title={emptyTitle}
        hint={emptyHint}
      >
        {#snippet action()}
          {#if !app.query.trim()}
            <button class="btn btn-primary" onclick={() => (sheet = 'newfolder')}>
              <Icon name="folder-plus" size={18} />
              {t('browser.createFolder')}
            </button>
          {/if}
        {/snippet}
      </EmptyState>
    {:else}
      <div class="canvas" style="height: {totalH}px">
        {#each windowItems as item (item.entry.path)}
          <div
            class="cell"
            style="transform: {position(item.index)}; width: {app.settings.view === 'grid'
              ? cellW
              : viewportW - pad * 2}px; height: {tileH}px"
            role="listitem"
          >
            <FileTile
              entry={item.entry}
              view={app.settings.view}
              index={item.index}
              selected={app.selected.has(item.entry.path)}
              {selecting}
              dropTarget={dropTarget === item.entry.path}
              onactivate={onActivate}
              onlongpress={onLongPress}
              onhover={onHover}
              onpointerdownitem={onPointerDownItem}
              onpointermoveitem={onPointerMoveItem}
              onpointerupitem={onPointerUpItem}
            />
          </div>
        {/each}
      </div>
      {#if app.query.trim() && app.searching}
        <div class="searching-row">
          <span class="dot"></span>
          {t('browser.searching')} · {t('browser.searchScanned', { count: app.searchScanned })}
        </div>
      {/if}
    {/if}
  </div>

  {#if selecting}
    <SelectionBar
      count={app.selected.size}
      onshare={shareSelection}
      ondelete={deleteSelection}
      onmore={() => {
        target = selectedEntries.length === 1 ? selectedEntries[0] : null;
        sheet = 'actions';
      }}
      onclear={() => app.clearSelection()}
    />
  {:else}
    <div class="fab-wrap">
      <button class="fab" aria-label={t('a11y.fab')} onclick={() => (sheet = 'fab')}>
        <Icon name="plus" size={26} stroke={2.2} />
      </button>
    </div>
  {/if}
</div>

{#if sheet === 'sort'}
  <SortFilterSheet
    filter={app.filter as FilterKey}
    onfilter={changeFilter}
    onclose={() => (sheet = null)}
  />
{/if}

{#if sheet === 'actions'}
  <ActionSheet
    title={selectedEntries.length > 1
      ? t('browser.selected', { count: selectedEntries.length })
      : (target?.name ?? app.currentName)}
    {actions}
    onselect={handleAction}
    onclose={() => (sheet = null)}
  />
{/if}

{#if sheet === 'fab'}
  <ActionSheet
    title={t('common.more')}
    actions={[
      { id: 'newfolder', label: t('browser.createFolder'), icon: 'folder-plus' },
      { id: 'newfile', label: t('browser.newFile'), icon: 'file-plus' },
      { id: 'selectall', label: t('actions.selectAll'), icon: 'square-check-big' },
      { id: 'sort', label: t('browser.sort'), icon: 'arrow-up-down' },
      { id: 'home', label: t('home.title'), icon: 'folder' },
      { id: 'settings', label: t('settings.title'), icon: 'settings' },
    ]}
    onselect={async (id) => {
      sheet = null;
      if (id === 'newfolder' || id === 'newfile') sheet = id;
      else if (id === 'selectall') app.selectAll(items);
      else if (id === 'sort') sheet = 'sort';
      else if (id === 'home') await app.goHome();
      else if (id === 'settings') await app.goToSettings();
    }}
    onclose={() => (sheet = null)}
  />
{/if}

{#if sheet === 'rename' && target}
  <PromptSheet
    title={t('actions.rename')}
    label={t('props.name')}
    initial={target.name}
    confirmLabel={t('common.save')}
    {busy}
    onclose={() => {
      sheet = null;
      target = null;
    }}
    onconfirm={doRename}
  />
{/if}

{#if sheet === 'newfolder'}
  <PromptSheet
    title={t('browser.createFolder')}
    label={t('browser.folderName')}
    initial={suggestName(items, 'Yeni klasör')}
    confirmLabel={t('common.save')}
    {busy}
    onclose={() => (sheet = null)}
    onconfirm={(name) => doCreate('folder', name)}
  />
{/if}

{#if sheet === 'newfile'}
  <PromptSheet
    title={t('browser.newFile')}
    label={t('browser.newFileName')}
    initial={suggestName(items, 'yeni dosya.txt')}
    confirmLabel={t('common.save')}
    {busy}
    onclose={() => (sheet = null)}
    onconfirm={(name) => doCreate('file', name)}
  />
{/if}

{#if sheet === 'zip'}
  <PromptSheet
    title={t('actions.zipCreate')}
    label={t('transfer.zipName')}
    initial={zipInitial}
    confirmLabel={t('common.save')}
    {busy}
    onclose={() => (sheet = null)}
    onconfirm={doZip}
  />
{/if}

{#if sheet === 'props' && target}
  <PropertiesSheet
    entry={target}
    onclose={() => {
      sheet = null;
      target = null;
    }}
    onopenwith={() => {
      sheet = null;
      if (target)
        void openWithFile(target.path).catch((error) => {
          app.fail(error);
        });
    }}
  />
{/if}

<style>
  .appbar {
    position: sticky;
    top: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 2px;
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
    background: color-mix(in srgb, var(--paper) 90%, transparent);
    backdrop-filter: blur(14px);
    transition: box-shadow var(--dur-fast) var(--ease-out);
  }
  .appbar.raised {
    box-shadow: 0 1px 0 var(--line);
  }
  .titles {
    flex: 1;
    min-width: 0;
    display: grid;
    padding-left: 2px;
  }
  .titles h1 {
    font-family: var(--font-display);
    font-size: 1.125rem;
    font-weight: 640;
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .titles span {
    font-size: 0.6875rem;
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
  }
  .searchbar {
    padding-bottom: var(--space-2);
  }
  .search-field {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 48px;
    padding: 0 var(--space-2) 0 var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--paper-raised);
    border: 1px solid var(--line-strong);
    color: var(--ink-3);
  }
  .search-field input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: 0;
    background: transparent;
    color: var(--ink);
    font-size: 0.9375rem;
  }
  .search-field input:focus {
    outline: none;
  }
  .icon-btn.small {
    width: 36px;
    height: 36px;
    min-width: 36px;
    min-height: 36px;
  }
  .tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: 0 var(--space-4) var(--space-2);
  }
  .chips {
    display: flex;
    gap: var(--space-2);
    overflow-x: auto;
    scrollbar-width: none;
    flex: 1;
  }
  .chips::-webkit-scrollbar {
    display: none;
  }
  .sort {
    flex: none;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior-y: contain;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    padding-bottom: calc(96px + var(--safe-bottom));
  }
  .content::-webkit-scrollbar {
    display: none;
  }
  .content.selection {
    padding-bottom: calc(88px + var(--safe-bottom));
  }
  .canvas {
    position: relative;
    width: 100%;
  }
  .cell {
    position: absolute;
    top: 0;
    left: 0;
    will-change: transform;
  }
  .skeletons {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: var(--space-3);
    padding: var(--space-4);
  }
  .sk {
    display: grid;
    gap: var(--space-2);
  }
  .sk .line {
    height: 10px;
    width: 70%;
  }
  .searching-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    justify-content: center;
    padding: var(--space-3);
    color: var(--ink-3);
    font-size: 0.75rem;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.25;
    }
  }
  .fab-wrap {
    position: fixed;
    right: max(var(--space-4), var(--safe-right));
    bottom: calc(var(--safe-bottom) + var(--space-4));
    z-index: 29;
  }
  .fab {
    width: 60px;
    height: 60px;
    border-radius: 20px;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--on-accent);
    box-shadow: var(--shadow-md);
    transition: transform var(--dur-fast) var(--ease-spring);
  }
  .fab:active {
    transform: scale(0.93);
  }
</style>
