<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    grouped?: boolean;
    inset?: boolean;
    children: Snippet;
  }

  let { grouped = true, inset = true, children }: Props = $props();
</script>

<div class="ios-list" class:grouped class:inset>
  {@render children()}
</div>

<style>
  .ios-list {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
  }

  .ios-list.grouped {
    background: transparent;
  }

  .ios-list.grouped.inset {
    padding: 0 var(--ios-grouped-inset, 16px);
  }

  .ios-list.grouped.inset > :global(.ios-list-section) {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-grouped-radius, 10px);
    overflow: hidden;
    margin-bottom: var(--ios-section-spacing, 35px);
  }

  .ios-list.grouped.inset > :global(.ios-list-section:last-child) {
    margin-bottom: 0;
  }
</style>
