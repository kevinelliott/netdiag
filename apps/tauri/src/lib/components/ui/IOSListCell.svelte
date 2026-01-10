<script lang="ts">
  import { Icon } from '../icons';
  import type { IconName } from '../icons';

  interface Props {
    title: string;
    subtitle?: string;
    value?: string;
    icon?: IconName;
    disclosure?: boolean;
    destructive?: boolean;
    href?: string;
    onclick?: () => void;
  }

  let {
    title,
    subtitle,
    value,
    icon,
    disclosure = false,
    destructive = false,
    href,
    onclick,
  }: Props = $props();
</script>

{#if href}
  <a {href} class="ios-list-cell" class:destructive class:has-icon={!!icon} {onclick}>
    {#if icon}
      <div class="cell-icon">
        <Icon name={icon} size={22} />
      </div>
    {/if}
    <div class="cell-content">
      <span class="cell-title">{title}</span>
      {#if subtitle}
        <span class="cell-subtitle">{subtitle}</span>
      {/if}
    </div>
    <div class="cell-accessory">
      {#if value}
        <span class="cell-value">{value}</span>
      {/if}
      {#if disclosure}
        <Icon name="chevron" size={14} color="var(--ios-gray-3, #C7C7CC)" />
      {/if}
    </div>
  </a>
{:else}
  <button
    class="ios-list-cell"
    class:destructive
    class:has-icon={!!icon}
    {onclick}
    type="button"
  >
    {#if icon}
      <div class="cell-icon">
        <Icon name={icon} size={22} />
      </div>
    {/if}
    <div class="cell-content">
      <span class="cell-title">{title}</span>
      {#if subtitle}
        <span class="cell-subtitle">{subtitle}</span>
      {/if}
    </div>
    <div class="cell-accessory">
      {#if value}
        <span class="cell-value">{value}</span>
      {/if}
      {#if disclosure}
        <Icon name="chevron" size={14} color="var(--ios-gray-3, #C7C7CC)" />
      {/if}
    </div>
  </button>
{/if}

<style>
  .ios-list-cell {
    display: flex;
    align-items: center;
    min-height: var(--ios-cell-min-height, 44px);
    padding: var(--ios-cell-padding-vertical, 11px) var(--ios-cell-padding-horizontal, 16px);
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border: none;
    text-decoration: none;
    color: inherit;
    width: 100%;
    text-align: left;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
    position: relative;
    gap: 12px;
  }

  /* Separator - aligned to text edge per iOS HIG */
  .ios-list-cell::after {
    content: '';
    position: absolute;
    left: var(--ios-cell-padding-horizontal, 16px);
    right: 0;
    bottom: 0;
    height: 0.5px;
    background: var(--ios-separator, rgba(60, 60, 67, 0.29));
  }

  /* Separator inset when cell has icon */
  .ios-list-cell.has-icon::after {
    left: calc(var(--ios-cell-padding-horizontal, 16px) + 22px + 12px);
  }

  /* Remove separator from last child */
  .ios-list-cell:last-child::after {
    display: none;
  }

  /* Active state */
  .ios-list-cell:active {
    background: var(--ios-fill-tertiary, rgba(120, 120, 128, 0.12));
  }

  .cell-icon {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ios-blue, #007AFF);
    flex-shrink: 0;
  }

  .cell-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .cell-title {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-body-size, 17px);
    font-weight: var(--ios-body-weight, 400);
    letter-spacing: var(--ios-body-tracking, -0.41px);
    line-height: var(--ios-body-leading, 22px);
    color: var(--ios-label-primary, #000000);
  }

  .ios-list-cell.destructive .cell-title {
    color: var(--ios-red, #FF3B30);
  }

  .cell-subtitle {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-caption1-size, 12px);
    font-weight: var(--ios-caption1-weight, 400);
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
    line-height: var(--ios-caption1-leading, 16px);
  }

  .cell-accessory {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .cell-value {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-body-size, 17px);
    font-weight: var(--ios-body-weight, 400);
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
  }
</style>
