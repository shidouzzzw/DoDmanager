<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { ICON_PATHS } from '../icons/paths';

  interface Props {
    name: string;
    size?: number;
    stroke?: number;
    label?: string;
    class?: string;
  }

  let { name, size = 22, stroke = 1.75, label, class: className = '' }: Props = $props();

  const content = $derived(ICON_PATHS[name] ?? ICON_PATHS['file']);
</script>

<svg
  class={className}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  role={label ? 'img' : 'presentation'}
  aria-label={label}
  aria-hidden={label ? undefined : 'true'}
>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- ikonlar derleme zamanında sabit, kullanıcı girdisi yok -->
  {@html content}
</svg>
