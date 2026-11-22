// SPDX-License-Identifier: GPL-3.0-or-later
import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';
import { setLocale } from './lib/i18n';

// Varsayılan dil Türkçe; kayıtlı ayar App açılışında uygulanır.
setLocale('tr');

const target = document.getElementById('app');
if (!target) throw new Error('#app bulunamadı');

const app = mount(App, { target });

export default app;
