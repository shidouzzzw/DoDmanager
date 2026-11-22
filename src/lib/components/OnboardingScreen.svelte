<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from '../i18n';
  import { pickLocation, requestAllFilesAccess, accessStatus } from '../services/settings';
  import { app } from '../state/app.svelte';
  import type { AccessMode } from '../types';
  import Icon from './Icon.svelte';

  let busy = $state(false);
  let chosen = $state<AccessMode | null>(null);
  let pickedFolder = $state<string | null>(null);

  async function finish(mode: AccessMode) {
    await app.updateSettings({ onboardingComplete: true, accessMode: mode });
    app.access = await accessStatus().catch(() => app.access);
    app.screen = 'home';
    await app.refreshHome();
    void app.checkIncoming();
  }

  async function chooseScoped() {
    busy = true;
    chosen = 'scoped';
    try {
      const location = await pickLocation();
      pickedFolder = location.label;
      await app.refreshLocations();
      await finish('scoped');
    } catch (error) {
      app.fail(error);
    } finally {
      busy = false;
    }
  }

  async function chooseAllFiles() {
    busy = true;
    chosen = 'all_files';
    try {
      await requestAllFilesAccess().catch(() => {});
      const status = await accessStatus().catch(() => null);
      app.access = status;
      if (status?.hasAllFilesAccess) {
        await finish('all_files');
      } else {
        app.toast(t('settings.grantAccess'));
        chosen = null;
      }
    } finally {
      busy = false;
    }
  }

  async function skip() {
    await finish('scoped');
  }
</script>

<div class="screen onboarding">
  <div class="scroll">
    <header class="hero">
      <div class="brand" aria-hidden="true">D</div>
      <h1>{t('onboarding.welcomeTitle')}</h1>
      <p>{t('onboarding.welcomeBody')}</p>
    </header>

    <div class="cards">
      <section class="option recommended" class:active={chosen === 'scoped'}>
        <div class="option-head">
          <span class="option-icon"><Icon name="shield-check" size={24} /></span>
          <span class="badge">{t('onboarding.scopedBest')}</span>
        </div>
        <h2>{t('onboarding.scopedTitle')}</h2>
        <p>{t('onboarding.scopedBody')}</p>
        <ul>
          <li><Icon name="check" size={15} /> {t('onboarding.scopedCap1')}</li>
          <li><Icon name="check" size={15} /> {t('onboarding.scopedCap2')}</li>
          <li><Icon name="check" size={15} /> {t('onboarding.scopedCap3')}</li>
        </ul>
        <button class="btn btn-primary btn-block" onclick={chooseScoped} disabled={busy}>
          <Icon name="folder-plus" size={18} />
          {pickedFolder ?? t('onboarding.pickFolder')}
        </button>
      </section>

      <section class="option" class:active={chosen === 'all_files'}>
        <div class="option-head">
          <span class="option-icon warn"><Icon name="hard-drive" size={24} /></span>
        </div>
        <h2>{t('onboarding.allFilesTitle')}</h2>
        <p>{t('onboarding.allFilesBody')}</p>
        <ul>
          <li><Icon name="check" size={15} /> {t('onboarding.allFilesCap1')}</li>
          <li><Icon name="check" size={15} /> {t('onboarding.allFilesCap2')}</li>
          <li><Icon name="check" size={15} /> {t('onboarding.allFilesCap3')}</li>
        </ul>
        <p class="warning">
          <Icon name="triangle-alert" size={15} />
          {t('onboarding.allFilesWarning')}
        </p>
        <button class="btn btn-secondary btn-block" onclick={chooseAllFiles} disabled={busy}>
          {t('onboarding.allFilesEnable')}
        </button>
      </section>
    </div>

    <button class="skip" onclick={skip} disabled={busy}>{t('onboarding.skip')}</button>

    <footer class="foot">
      <Icon name="wifi-off" size={16} />
      <span>{t('settings.privacyBody')}</span>
    </footer>
  </div>
</div>

<style>
  .onboarding {
    background:
      radial-gradient(120% 60% at 90% -10%, var(--accent-soft), transparent 55%), var(--paper);
  }
  .scroll {
    padding-bottom: calc(var(--safe-bottom) + var(--space-5));
  }
  .hero {
    padding: calc(var(--safe-top) + var(--space-6)) var(--space-5) var(--space-5);
    display: grid;
    gap: var(--space-3);
  }
  .brand {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    background: var(--ink);
    color: var(--paper);
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 1.5rem;
    box-shadow: var(--shadow-md);
  }
  .hero h1 {
    font-size: clamp(1.9rem, 8vw, 2.4rem);
    line-height: 1.05;
    letter-spacing: -0.035em;
    font-weight: 680;
  }
  .hero p {
    color: var(--ink-3);
    font-size: 0.9375rem;
    line-height: 1.55;
    max-width: 36ch;
  }
  .cards {
    display: grid;
    gap: var(--space-3);
    padding: 0 var(--space-4);
  }
  .option {
    border-radius: var(--radius-xl);
    border: 1px solid var(--line);
    background: var(--paper-raised);
    padding: var(--space-4);
    display: grid;
    gap: var(--space-2);
    box-shadow: var(--shadow-sm);
    transition:
      transform var(--dur-fast) var(--ease-spring),
      box-shadow var(--dur-fast) var(--ease-out);
  }
  .option:active {
    transform: scale(0.99);
  }
  .option.active {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .option-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .option-icon {
    width: 48px;
    height: 48px;
    border-radius: 15px;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .option-icon.warn {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .option h2 {
    font-size: 1.125rem;
    font-weight: 650;
  }
  .option p {
    font-size: 0.8125rem;
    color: var(--ink-3);
    line-height: 1.5;
  }
  .option ul {
    display: grid;
    gap: var(--space-1);
    margin: var(--space-1) 0 var(--space-2);
  }
  .option li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.8125rem;
    color: var(--ink-2);
  }
  .option li :global(svg) {
    color: var(--accent);
    flex: none;
  }
  .warning {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    background: var(--paper-sunken);
    border-radius: var(--radius-sm);
    padding: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .warning :global(svg) {
    color: var(--danger);
    flex: none;
    margin-top: 2px;
  }
  .skip {
    display: block;
    margin: var(--space-4) auto 0;
    min-height: 48px;
    padding: 0 var(--space-4);
    color: var(--ink-3);
    font-size: 0.875rem;
    text-decoration: underline;
    text-underline-offset: 4px;
  }
  .foot {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    max-width: 42ch;
    margin: var(--space-2) auto 0;
    padding: 0 var(--space-5);
    color: var(--ink-3);
    font-size: 0.6875rem;
    line-height: 1.5;
  }
</style>
