<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { t } from './lib/i18n';
  import { onJobFinished, onJobProgress } from './lib/services/jobs';
  import { app } from './lib/state/app.svelte';
  import BrowserScreen from './lib/components/BrowserScreen.svelte';
  import ErrorSheet from './lib/components/ErrorSheet.svelte';
  import FolderPickerScreen from './lib/components/FolderPickerScreen.svelte';
  import HomeScreen from './lib/components/HomeScreen.svelte';
  import IncomingSheet from './lib/components/IncomingSheet.svelte';
  import JobBanner from './lib/components/JobBanner.svelte';
  import OnboardingScreen from './lib/components/OnboardingScreen.svelte';
  import PreviewScreen from './lib/components/PreviewScreen.svelte';
  import SettingsScreen from './lib/components/SettingsScreen.svelte';
  import Snackbar from './lib/components/Snackbar.svelte';
  import TrashScreen from './lib/components/TrashScreen.svelte';
  import { accessStatus, openAppSettings, requestAllFilesAccess } from './lib/services/settings';
  import { cancelJob } from './lib/services/files';

  let unlisten: (() => void)[] = [];
  let mediaQuery: MediaQueryList | null = null;
  let viewportListener: (() => void) | null = null;

  onMount(async () => {
    await app.init();

    // Android geri tuşu ve hareketi: MainActivity bu köprüyü çağırır.
    (window as unknown as { __dodBack?: () => boolean }).__dodBack = () => app.back();

    const jobProgress = await onJobProgress((progress) => {
      app.job = progress;
      app.jobKind = progress.kind;
    });
    const jobFinished = await onJobFinished((finished) => {
      app.job = null;
      if (!finished.ok && finished.code && finished.code !== 'CANCELLED') {
        app.fail({ code: finished.code, message: finished.message ?? '' });
      }
    });
    unlisten.push(jobProgress, jobFinished);

    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const onScheme = () => {
      if (app.settings.theme === 'system') void app.applyAppearance();
    };
    mediaQuery.addEventListener('change', onScheme);
    unlisten.push(() => mediaQuery?.removeEventListener('change', onScheme));

    // Klavye açılışında alt çubuklar görünür kalsın.
    const viewport = window.visualViewport;
    if (viewport) {
      const onResize = () => {
        const inset = Math.max(0, window.innerHeight - viewport.height - viewport.offsetTop);
        document.documentElement.style.setProperty('--kb-inset', `${inset}px`);
      };
      viewport.addEventListener('resize', onResize);
      viewportListener = () => viewport.removeEventListener('resize', onResize);
      onResize();
    }

    // Sistem ayarlarından dönünce izin durumunu tazele.
    const onVisible = async () => {
      if (document.visibilityState !== 'visible') return;
      app.access = await accessStatus().catch(() => app.access);
      await app.refreshLocations();
      await app.checkIncoming();
    };
    document.addEventListener('visibilitychange', onVisible);
    unlisten.push(() => document.removeEventListener('visibilitychange', onVisible));
  });

  onDestroy(() => {
    for (const off of unlisten) off();
    viewportListener?.();
  });

  function jobLabel(kind: string): string {
    const map: Record<string, string> = {
      copy: 'job.copying',
      move: 'job.moving',
      delete: 'job.deleting',
      zip: 'job.zipping',
      unzip: 'job.unzipping',
    };
    return t(map[kind] ?? 'job.done');
  }

  async function handleErrorAction(action: string) {
    const error = app.error;
    app.dismissError();
    if (action === 'settings') {
      if (error?.code === 'PERMISSION_DENIED' && app.settings.accessMode === 'scoped') {
        await app.goToSettings();
      } else {
        await openAppSettings().catch(() => {});
        await requestAllFilesAccess().catch(() => {});
      }
    } else if (action === 'retry') {
      await app.reload();
    } else if (action === 'choose_location') {
      await app.pickNewLocation();
    } else if (action === 'rename') {
      app.screen = 'browser';
    }
  }
</script>

{#if !app.ready}
  <div class="boot" aria-live="polite">
    <div class="mark">D</div>
    <p>{t('common.loading')}</p>
  </div>
{:else}
  {#key app.screen}
    <div class="screen-wrap" in:fade={{ duration: app.settings.reducedMotion ? 0 : 160 }}>
      {#if app.screen === 'onboarding'}
        <OnboardingScreen />
      {:else if app.screen === 'home'}
        <HomeScreen />
      {:else if app.screen === 'browser'}
        <BrowserScreen />
      {:else if app.screen === 'settings'}
        <SettingsScreen />
      {:else if app.screen === 'trash'}
        <TrashScreen />
      {:else if app.screen === 'preview'}
        <PreviewScreen />
      {:else if app.screen === 'picker'}
        <FolderPickerScreen />
      {/if}
    </div>
  {/key}
{/if}

{#if app.error}
  <ErrorSheet error={app.error} onclose={() => app.dismissError()} onaction={handleErrorAction} />
{/if}

{#if app.incoming.length}
  <IncomingSheet
    files={app.incoming}
    onsave={() => {
      app.beginTransfer(
        'copy',
        app.incoming.map((file) => file.uri),
      );
    }}
    ondismiss={() => app.dismissIncoming()}
  />
{/if}

{#if app.job}
  <JobBanner
    job={app.job}
    label={jobLabel(app.jobKind)}
    oncancel={() => cancelJob(app.job?.jobId ?? '').catch(() => {})}
    offset={app.selected.size > 0 ? 72 : 0}
  />
{/if}

{#if app.snackbar}
  <Snackbar
    message={app.snackbar.message}
    undo={app.snackbar.undo}
    offset={app.job ? 86 : app.selected.size > 0 ? 72 : 0}
    onclose={() => app.dismissSnackbar()}
    onundo={async () => {
      const undo = app.snackbar?.undo;
      app.dismissSnackbar();
      if (undo) await undo();
    }}
  />
{/if}

<style>
  .boot {
    min-height: 100dvh;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: var(--space-3);
    color: var(--ink-3);
    font-size: 0.8125rem;
  }
  .mark {
    width: 60px;
    height: 60px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    background: var(--ink);
    color: var(--paper);
    font-family: var(--font-display);
    font-size: 1.6rem;
    font-weight: 700;
    animation: pop 400ms var(--ease-spring);
  }
  .screen-wrap {
    min-height: 100dvh;
  }
</style>
