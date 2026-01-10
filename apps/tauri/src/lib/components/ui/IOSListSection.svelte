<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    header?: string;
    footer?: string;
    children: Snippet;
  }

  let { header, footer, children }: Props = $props();
</script>

<div class="ios-list-section">
  {#if header}
    <div class="section-header">{header}</div>
  {/if}
  <div class="section-content">
    {@render children()}
  </div>
  {#if footer}
    <div class="section-footer">{footer}</div>
  {/if}
</div>

<style>
  .ios-list-section {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-grouped-radius, 10px);
    overflow: hidden;
  }

  .section-header {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-footnote-size, 13px);
    font-weight: 400;
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
    text-transform: uppercase;
    padding: 0 var(--ios-cell-padding-horizontal, 16px);
    padding-top: 24px;
    padding-bottom: 8px;
    letter-spacing: -0.08px;
  }

  .section-content {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-grouped-radius, 10px);
    overflow: hidden;
  }

  .section-footer {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-footnote-size, 13px);
    font-weight: 400;
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
    padding: 8px var(--ios-cell-padding-horizontal, 16px);
    padding-bottom: 0;
    letter-spacing: -0.08px;
  }
</style>
