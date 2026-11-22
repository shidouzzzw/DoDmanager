<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import BottomSheet from './BottomSheet.svelte';
  import Icon from './Icon.svelte';

  export interface SheetAction {
    id: string;
    label: string;
    icon: string;
    hint?: string;
    danger?: boolean;
  }

  interface Props {
    title: string;
    actions: SheetAction[];
    onselect: (id: string) => void;
    onclose: () => void;
  }

  let { title, actions, onselect, onclose }: Props = $props();
</script>

<BottomSheet {title} {onclose}>
  {#each actions as action (action.id)}
    <button class="menu-item" class:danger={action.danger} onclick={() => onselect(action.id)}>
      <Icon name={action.icon} size={21} stroke={1.6} />
      <span class="text">
        {action.label}
        {#if action.hint}<small>{action.hint}</small>{/if}
      </span>
    </button>
  {/each}
</BottomSheet>

<style>
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
</style>
