// SPDX-License-Identifier: GPL-3.0-or-later
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    // Erişilebilirlik uyarıları CI'da hata sayılır.
    warningFilter: (warning) => !warning.code.startsWith('a11y'),
  },
};
