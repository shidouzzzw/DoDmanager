// SPDX-License-Identifier: GPL-3.0-or-later
// Lucide ikon setinden yalnızca kullandığımız çizimleri üretir.
// Kullanım: npm pack lucide-static && node scripts/build-icons.mjs <paket-klasörü>
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const ICONS = [
  'folder',
  'folder-open',
  'folder-plus',
  'folder-input',
  'file',
  'file-plus',
  'file-text',
  'file-code',
  'image',
  'film',
  'video',
  'music',
  'archive',
  'package',
  'file-question',
  'layout-grid',
  'list',
  'search',
  'x',
  'chevron-left',
  'chevron-right',
  'chevron-down',
  'arrow-left',
  'arrow-up',
  'arrow-down',
  'more-vertical',
  'star',
  'trash-2',
  'settings',
  'plus',
  'check',
  'check-circle',
  'share-2',
  'pencil',
  'copy',
  'info',
  'hard-drive',
  'arrow-up-down',
  'arrow-down-wide-narrow',
  'filter',
  'play',
  'pause',
  'lock',
  'triangle-alert',
  'circle-alert',
  'undo-2',
  'refresh-cw',
  'menu',
  'external-link',
  'sun',
  'moon',
  'monitor',
  'palette',
  'languages',
  'sparkles',
  'eye',
  'eye-off',
  'type',
  'shield-check',
  'circle-help',
  'folder-heart',
  'rotate-cw',
  'arrow-up-right',
  'move',
  'grip-vertical',
  'zoom-in',
  'book-open',
  'wifi-off',
  'image-off',
  'loader-circle',
  'square-check-big',
  'ellipsis',
];

const source = process.argv[2] || '/tmp/lucidepack/package/icons';
const paths = {};

for (const name of ICONS) {
  const svg = readFileSync(join(source, `${name}.svg`), 'utf8');
  paths[name] = svg
    .replace(/^[\s\S]*?<svg[^>]*>/, '')
    .replace(/<\/svg>[\s\S]*$/, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

const header = `// SPDX-License-Identifier: ISC
// Lucide ikon setinden üretildi (https://lucide.dev, ISC).
// \`scripts/build-icons.mjs\` ile yeniden üretilebilir.

export const ICON_PATHS: Record<string, string> = `;

writeFileSync('src/lib/icons/paths.ts', `${header}${JSON.stringify(paths, null, 1)} as const;\n`);
console.log(`${Object.keys(paths).length} ikon yazıldı.`);
