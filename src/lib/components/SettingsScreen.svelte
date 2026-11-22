<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { SUPPORTED_LOCALES, t, type Locale } from '../i18n';
  import { formatSize } from '../services/format';
  import {
    accessStatus,
    dynamicAccent,
    openAppSettings,
    openExternal,
    requestAllFilesAccess,
    setAccessMode,
  } from '../services/settings';
  import { app } from '../state/app.svelte';
  import type { AccessMode, ThemeMode } from '../types';
  import Icon from './Icon.svelte';

  const themes: { key: ThemeMode; label: string; icon: string }[] = [
    { key: 'system', label: t('settings.themeSystem'), icon: 'monitor' },
    { key: 'light', label: t('settings.themeLight'), icon: 'sun' },
    { key: 'dark', label: t('settings.themeDark'), icon: 'moon' },
  ];

  const accents = [
    { key: 'indigo', color: '#34499b' },
    { key: 'ink', color: '#26313e' },
    { key: 'moss', color: '#2f5f43' },
    { key: 'amber', color: '#8f5a12' },
    { key: 'clay', color: '#8c4535' },
    { key: 'plum', color: '#613a6b' },
  ];

  let busy = $state(false);
  let refreshing = $state(false);

  onMount(() => {
    void app.refreshHome();
  });

  async function toggle(key: 'haptics' | 'showHidden' | 'reducedMotion', value: boolean) {
    if (key === 'haptics') await app.updateSettings({ haptics: value });
    else if (key === 'showHidden') await app.updateSettings({ showHidden: value });
    else await app.updateSettings({ reducedMotion: value });
  }

  async function enableDynamicColor(value: boolean) {
    if (!value) {
      await app.updateSettings({ dynamicColor: false });
      return;
    }
    app.dynamicColorValue = null;
    const color = await dynamicAccent().catch(() => null);
    if (!color) {
      app.toast(t('settings.dynamicUnavailable'));
      return;
    }
    app.dynamicColorValue = color;
    await app.updateSettings({ dynamicColor: true });
  }

  async function changeMode(mode: AccessMode) {
    if (mode === app.settings.accessMode) return;
    if (mode === 'all_files' && !app.access?.hasAllFilesAccess) {
      await requestAllFilesAccess().catch(() => {});
      app.toast(t('settings.grantAccess'));
      return;
    }
    busy = true;
    try {
      const status = await setAccessMode(mode);
      app.access = status;
      app.settings = { ...app.settings, accessMode: mode };
      await app.refreshLocations();
      await app.refreshHome();
      app.toast(t('settings.modeChanged'));
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function refreshAccess() {
    refreshing = true;
    try {
      app.access = await accessStatus();
      await app.refreshLocations();
    } catch (error) {
      app.fail(error);
    } finally {
      refreshing = false;
    }
  }

  async function changeLocale(locale: Locale) {
    await app.updateSettings({ locale });
  }

  async function resetOnboarding() {
    await app.updateSettings({ onboardingComplete: false });
    app.screen = 'onboarding';
  }

  const accessLabel = $derived(
    app.settings.accessMode === 'all_files'
      ? t('settings.accessAllFiles')
      : t('settings.accessScoped'),
  );
</script>

<div class="screen">
  <header class="appbar pad-x">
    <button class="icon-btn" aria-label={t('a11y.back')} onclick={() => app.back()}>
      <Icon name="chevron-left" size={24} />
    </button>
    <h1>{t('settings.title')}</h1>
  </header>

  <div class="scroll">
    <section class="group">
      <h2 class="section-title">{t('settings.appearance')}</h2>
      <div class="card">
        <div class="row-stack">
          <span class="row-label">{t('settings.theme')}</span>
          <div class="segmented" role="group" aria-label={t('settings.theme')}>
            {#each themes as theme (theme.key)}
              <button
                class="segment"
                class:active={app.settings.theme === theme.key}
                aria-pressed={app.settings.theme === theme.key}
                onclick={() => app.updateSettings({ theme: theme.key })}
              >
                <Icon name={theme.icon} size={17} />
                {theme.label}
              </button>
            {/each}
          </div>
        </div>

        <hr class="rule" />

        <div class="row-stack">
          <span class="row-label">{t('settings.accent')}</span>
          <div class="swatches" role="group" aria-label={t('settings.accent')}>
            {#each accents as accent (accent.key)}
              <button
                class="swatch"
                class:active={app.settings.accent === accent.key}
                style="--swatch: {accent.color}"
                aria-label={accent.key}
                aria-pressed={app.settings.accent === accent.key}
                onclick={() => app.updateSettings({ accent: accent.key, dynamicColor: false })}
              >
                {#if app.settings.accent === accent.key}
                  <Icon name="check" size={16} stroke={2.6} />
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <hr class="rule" />

        <label class="row-toggle">
          <span class="row-text">
            <span class="row-label">{t('settings.dynamicColor')}</span>
            <span class="row-hint">{t('settings.dynamicColorHint')}</span>
          </span>
          <input
            type="checkbox"
            class="switch"
            checked={app.settings.dynamicColor}
            onchange={(event) =>
              enableDynamicColor((event.currentTarget as HTMLInputElement).checked)}
          />
        </label>

        <hr class="rule" />

        <div class="row-stack">
          <span class="row-label">{t('settings.language')}</span>
          <div class="chips">
            {#each SUPPORTED_LOCALES as locale (locale.code)}
              <button
                class="chip"
                class:is-active={app.settings.locale === locale.code}
                aria-pressed={app.settings.locale === locale.code}
                onclick={() => changeLocale(locale.code)}
              >
                {locale.name}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <section class="group">
      <h2 class="section-title">{t('settings.behaviour')}</h2>
      <div class="card">
        <label class="row-toggle">
          <span class="row-text">
            <span class="row-label">{t('settings.haptics')}</span>
            <span class="row-hint">{t('settings.hapticsHint')}</span>
          </span>
          <input
            type="checkbox"
            class="switch"
            checked={app.settings.haptics}
            onchange={(event) =>
              toggle('haptics', (event.currentTarget as HTMLInputElement).checked)}
          />
        </label>
        <hr class="rule" />
        <label class="row-toggle">
          <span class="row-text">
            <span class="row-label">{t('settings.showHidden')}</span>
            <span class="row-hint">{t('settings.showHiddenHint')}</span>
          </span>
          <input
            type="checkbox"
            class="switch"
            checked={app.settings.showHidden}
            onchange={(event) =>
              toggle('showHidden', (event.currentTarget as HTMLInputElement).checked)}
          />
        </label>
        <hr class="rule" />
        <label class="row-toggle">
          <span class="row-text">
            <span class="row-label">{t('settings.reducedMotion')}</span>
            <span class="row-hint">{t('settings.reducedMotionHint')}</span>
          </span>
          <input
            type="checkbox"
            class="switch"
            checked={app.settings.reducedMotion}
            onchange={(event) =>
              toggle('reducedMotion', (event.currentTarget as HTMLInputElement).checked)}
          />
        </label>
        <hr class="rule" />
        <div class="row-stack">
          <span class="row-label">{t('settings.columns')}</span>
          <div class="stepper">
            <button
              class="icon-btn"
              aria-label={t('settings.columns')}
              onclick={() =>
                app.updateSettings({ gridColumns: Math.max(2, app.settings.gridColumns - 1) })}
            >
              <Icon name="chevron-left" size={18} />
            </button>
            <strong>{app.settings.gridColumns}</strong>
            <button
              class="icon-btn"
              aria-label={t('settings.columns')}
              onclick={() =>
                app.updateSettings({ gridColumns: Math.min(5, app.settings.gridColumns + 1) })}
            >
              <Icon name="chevron-right" size={18} />
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="group">
      <h2 class="section-title">{t('settings.access')}</h2>
      <div class="mode">
        <span class="mode-badge">
          <Icon
            name={app.settings.accessMode === 'all_files' ? 'hard-drive' : 'shield-check'}
            size={14}
          />
          {accessLabel}
        </span>
        <p class="mode-status">
          {#if app.settings.accessMode === 'all_files'}
            {app.access?.hasAllFilesAccess
              ? t('settings.permissionGranted')
              : t('settings.permissionMissing')}
          {:else}
            {t('settings.accessScopedHint')}
          {/if}
        </p>
      </div>

      <button
        class="mode-card"
        class:active={app.settings.accessMode === 'scoped'}
        onclick={() => changeMode('scoped')}
        disabled={busy}
      >
        <span class="mode-icon"><Icon name="shield-check" size={22} /></span>
        <span class="mode-text">
          <strong>{t('settings.accessScoped')}</strong>
          <small>{t('settings.accessScopedHint')}</small>
        </span>
        {#if app.settings.accessMode === 'scoped'}<Icon name="check-circle" size={20} />{/if}
      </button>

      <button
        class="mode-card"
        class:active={app.settings.accessMode === 'all_files'}
        onclick={() => changeMode('all_files')}
        disabled={busy}
      >
        <span class="mode-icon"><Icon name="hard-drive" size={22} /></span>
        <span class="mode-text">
          <strong>{t('settings.accessAllFiles')}</strong>
          <small>{t('settings.accessAllFilesHint')}</small>
        </span>
        {#if app.settings.accessMode === 'all_files'}<Icon name="check-circle" size={20} />{/if}
      </button>

      <div class="access-actions">
        <button class="btn btn-secondary" onclick={refreshAccess} disabled={refreshing}>
          <Icon name="refresh-cw" size={17} />
          {refreshing ? t('common.loading') : t('common.refresh')}
        </button>
        <button class="btn btn-ghost" onclick={() => requestAllFilesAccess().catch(() => {})}>
          <Icon name="external-link" size={17} />
          {t('settings.grantAccess')}
        </button>
        <button class="btn btn-ghost" onclick={() => openAppSettings().catch(() => {})}>
          <Icon name="settings" size={17} />
          {t('settings.title')}
        </button>
      </div>
    </section>

    <section class="group">
      <h2 class="section-title">{t('settings.storage')}</h2>
      <div class="card">
        <div class="row-between">
          <span class="row-label">{t('common.used')}</span>
          <span class="value">{app.storage ? formatSize(app.storage.used) : '—'}</span>
        </div>
        <hr class="rule" />
        <div class="row-between">
          <span class="row-label">{t('common.free')}</span>
          <span class="value">{app.storage ? formatSize(app.storage.free) : '—'}</span>
        </div>
        <hr class="rule" />
        <button class="row-button" onclick={() => app.goToTrash()}>
          <Icon name="trash-2" size={18} />
          <span>{t('settings.trash')}</span>
          <span class="value">{formatSize(app.trashBytes)}</span>
          <Icon name="chevron-right" size={18} />
        </button>
      </div>
    </section>

    <section class="group">
      <h2 class="section-title">{t('settings.about')}</h2>
      <div class="card">
        <div class="row-between">
          <span class="row-label">{t('settings.version')}</span>
          <span class="value">{app.info?.version ?? '0.1.0'}</span>
        </div>
        <hr class="rule" />
        <button
          class="row-button"
          onclick={() => openExternal('https://www.gnu.org/licenses/gpl-3.0.html').catch(() => {})}
        >
          <Icon name="book-open" size={18} />
          <span>{t('settings.license')}</span>
          <span class="value">GPL-3.0-or-later</span>
          <Icon name="external-link" size={16} />
        </button>
        <hr class="rule" />
        <button
          class="row-button"
          onclick={() => openExternal('https://github.com/shidouzzzw/DoDmanager').catch(() => {})}
        >
          <Icon name="file-code" size={18} />
          <span>{t('settings.sourceCode')}</span>
          <span class="value">GitHub</span>
          <Icon name="external-link" size={16} />
        </button>
        <hr class="rule" />
        <div class="privacy">
          <Icon name="wifi-off" size={18} />
          <span>{t('settings.privacyBody')}</span>
        </div>
        <hr class="rule" />
        <button class="row-button" onclick={resetOnboarding}>
          <Icon name="sparkles" size={18} />
          <span>{t('settings.resetOnboarding')}</span>
          <Icon name="chevron-right" size={18} />
        </button>
      </div>
    </section>

    <footer class="foot pad-x">
      <span>Dosya Dolabı · GPL-3.0-or-later · {app.info?.platform ?? 'android'}</span>
    </footer>
  </div>
</div>

<style>
  .appbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: calc(var(--header-height) + var(--safe-top));
    padding-top: var(--safe-top);
    border-bottom: 1px solid var(--line);
  }
  .appbar h1 {
    font-family: var(--font-display);
    font-size: 1.25rem;
    font-weight: 650;
  }
  .scroll {
    padding-bottom: calc(var(--safe-bottom) + var(--space-6));
  }
  .group {
    padding: var(--space-4) var(--space-4) 0;
  }
  .group .section-title {
    margin-bottom: var(--space-2);
  }
  .card {
    padding: var(--space-1) 0;
  }
  .row-stack {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
  }
  .row-label {
    font-size: 0.9375rem;
    font-weight: 560;
  }
  .row-hint {
    font-size: 0.75rem;
    color: var(--ink-3);
    line-height: 1.45;
  }
  .segmented {
    display: flex;
    background: var(--paper-sunken);
    border-radius: var(--radius-pill);
    padding: 3px;
    gap: 2px;
  }
  .segment {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 42px;
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-size: 0.8125rem;
    font-weight: 550;
    transition:
      background-color var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }
  .segment.active {
    background: var(--paper-raised);
    color: var(--ink);
    box-shadow: var(--shadow-sm);
  }
  .swatches {
    display: flex;
    gap: var(--space-2);
    flex-wrap: wrap;
  }
  .swatch {
    width: 44px;
    height: 44px;
    min-width: 44px;
    min-height: 44px;
    border-radius: 14px;
    background: var(--swatch);
    color: #fbf8f0;
    display: grid;
    place-items: center;
    border: 2px solid transparent;
  }
  .swatch.active {
    border-color: var(--ink);
    outline: 2px solid var(--paper);
    outline-offset: -5px;
  }
  .row-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 64px;
    padding: var(--space-2) var(--space-4);
  }
  .row-text {
    flex: 1;
    display: grid;
    gap: 2px;
  }
  .chips {
    display: flex;
    gap: var(--space-2);
    flex-wrap: wrap;
  }
  .switch {
    appearance: none;
    width: 52px;
    height: 32px;
    min-width: 52px;
    border-radius: 999px;
    background: var(--paper-sunken);
    border: 1px solid var(--line-strong);
    position: relative;
    transition: background-color var(--dur-fast) var(--ease-out);
    flex: none;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--paper-raised);
    box-shadow: var(--shadow-sm);
    transition: transform var(--dur-base) var(--ease-spring);
  }
  .switch:checked {
    background: var(--accent);
    border-color: transparent;
  }
  .switch:checked::after {
    transform: translateX(20px);
    background: var(--on-accent);
  }
  .stepper {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: var(--paper-sunken);
    border-radius: var(--radius-pill);
    padding: 2px;
    align-self: start;
  }
  .stepper strong {
    min-width: 20px;
    text-align: center;
  }
  .mode {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-3) var(--space-4);
    margin-bottom: var(--space-2);
    border-left: 3px solid var(--accent);
    background: var(--accent-soft);
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
  }
  .mode-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    font-weight: 650;
    color: var(--accent);
    letter-spacing: 0.02em;
  }
  .mode-status {
    font-size: 0.75rem;
    color: var(--ink-2);
  }
  .mode-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3);
    border-radius: var(--radius-lg);
    border: 1px solid var(--line);
    background: var(--paper-raised);
    margin-bottom: var(--space-2);
    text-align: left;
  }
  .mode-card.active {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .mode-icon {
    width: 44px;
    height: 44px;
    border-radius: 13px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .mode-text {
    flex: 1;
    display: grid;
    gap: 2px;
  }
  .mode-text strong {
    font-size: 0.9375rem;
  }
  .mode-text small {
    font-size: 0.75rem;
    color: var(--ink-3);
    line-height: 1.4;
  }
  .access-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }
  .row-between,
  .row-button {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 52px;
    padding: 0 var(--space-4);
  }
  .row-between .row-label {
    flex: 1;
  }
  .row-button {
    width: 100%;
    text-align: left;
    font-size: 0.9375rem;
  }
  .row-button span:first-of-type {
    flex: 1;
  }
  .row-button:active {
    background: var(--paper-sunken);
  }
  .value {
    color: var(--ink-3);
    font-size: 0.8125rem;
    font-variant-numeric: tabular-nums;
  }
  .privacy {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    padding: var(--space-3) var(--space-4);
    font-size: 0.75rem;
    color: var(--ink-3);
    line-height: 1.5;
  }
  .foot {
    padding-top: var(--space-5);
    color: var(--ink-3);
    font-size: 0.6875rem;
    text-align: center;
  }
</style>
