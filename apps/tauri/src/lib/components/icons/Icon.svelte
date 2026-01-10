<script lang="ts">
  import { iconPaths, type IconName } from './iconPaths';

  interface Props {
    name: IconName;
    size?: number;
    filled?: boolean;
    color?: string;
    strokeWidth?: number;
    class?: string;
  }

  let {
    name,
    size = 24,
    filled = false,
    color = 'currentColor',
    strokeWidth = 1.5,
    class: className = '',
  }: Props = $props();

  const icon = $derived(iconPaths[name]);
  const path = $derived(icon ? (filled ? icon.filled : icon.outline) : '');
</script>

{#if icon}
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke={color}
    stroke-width={strokeWidth}
    stroke-linecap="round"
    stroke-linejoin="round"
    class="icon {className}"
    aria-hidden="true"
  >
    <path d={path} />
  </svg>
{/if}

<style>
  .icon {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }
</style>
