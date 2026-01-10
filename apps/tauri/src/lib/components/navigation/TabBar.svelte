<script lang="ts">
  import { page } from '$app/stores';
  import { getPlatform } from '$lib/platform';
  import { Icon } from '../icons';
  import type { IconName } from '../icons';

  const platform = getPlatform();

  let showActionSheet = $state(false);

  interface Tab {
    href?: string;
    label: string;
    icon: IconName;
    isMenu?: boolean;
  }

  const tabs: Tab[] = [
    { href: '/', label: 'Dashboard', icon: 'dashboard' },
    { href: '/diagnose', label: 'Diagnose', icon: 'diagnose' },
    { href: '/wifi', label: 'WiFi', icon: 'wifi' },
    { href: '/speed', label: 'Speed', icon: 'speed' },
    { label: 'More', icon: 'more', isMenu: true },
  ];

  const moreItems: Tab[] = [
    { href: '/interfaces', label: 'Interfaces', icon: 'interfaces' },
    { href: '/ping', label: 'Ping', icon: 'ping' },
    { href: '/traceroute', label: 'Traceroute', icon: 'traceroute' },
    { href: '/dns', label: 'DNS', icon: 'dns' },
    { href: '/capture', label: 'Capture', icon: 'capture' },
    { href: '/report', label: 'Report', icon: 'report' },
    { href: '/fix', label: 'Fix', icon: 'fix' },
  ];

  function isActiveMoreItem(): boolean {
    return moreItems.some((item) => $page.url.pathname === item.href);
  }

  function handleTabTap(isMenu: boolean) {
    if (isMenu) {
      showActionSheet = !showActionSheet;
    } else {
      showActionSheet = false;
    }
  }

  function closeActionSheet() {
    showActionSheet = false;
  }

  function handleItemClick() {
    showActionSheet = false;
  }
</script>

<nav
  class="tabbar"
  class:ios={platform?.platform === 'ios'}
  class:android={platform?.platform === 'android'}
>
  {#each tabs as tab}
    {#if tab.isMenu}
      <button
        class="tabbar-item"
        class:active={isActiveMoreItem() || showActionSheet}
        onclick={() => handleTabTap(true)}
        type="button"
      >
        <div class="tabbar-icon">
          <Icon
            name={tab.icon}
            size={28}
            filled={isActiveMoreItem() || showActionSheet}
          />
        </div>
        <span class="tabbar-label">{tab.label}</span>
      </button>
    {:else if tab.href}
      <a
        href={tab.href}
        class="tabbar-item"
        class:active={$page.url.pathname === tab.href}
        onclick={() => handleTabTap(false)}
      >
        <div class="tabbar-icon">
          <Icon
            name={tab.icon}
            size={28}
            filled={$page.url.pathname === tab.href}
          />
        </div>
        <span class="tabbar-label">{tab.label}</span>
      </a>
    {/if}
  {/each}
</nav>

{#if showActionSheet}
  <!-- Backdrop -->
  <button
    class="action-sheet-backdrop"
    onclick={closeActionSheet}
    aria-label="Close menu"
    tabindex="-1"
  ></button>

  <!-- iOS Action Sheet -->
  <div class="action-sheet" class:ios={platform?.platform === 'ios'}>
    <div class="action-sheet-content">
      <div class="action-sheet-header">More</div>
      <div class="action-sheet-grid">
        {#each moreItems as item}
          <a
            href={item.href}
            class="action-item"
            class:active={$page.url.pathname === item.href}
            onclick={handleItemClick}
          >
            <div class="action-icon">
              <Icon
                name={item.icon}
                size={28}
                filled={$page.url.pathname === item.href}
              />
            </div>
            <span class="action-label">{item.label}</span>
          </a>
        {/each}
      </div>
    </div>
    <button class="cancel-button" onclick={closeActionSheet} type="button">
      Cancel
    </button>
  </div>
{/if}

<style>
  /* =================================
     TAB BAR - BASE
     ================================= */
  .tabbar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 56px;
    background: var(--bg-secondary);
    border-top: 0.5px solid var(--border);
    display: flex;
    justify-content: space-around;
    align-items: flex-start;
    padding-top: 6px;
    z-index: 100;
  }

  /* =================================
     TAB BAR - iOS
     ================================= */
  .tabbar.ios {
    height: calc(var(--ios-tabbar-height, 49px) + var(--safe-area-inset-bottom, 34px));
    padding-bottom: var(--safe-area-inset-bottom, 34px);
    background: var(--ios-tabbar-bg, rgba(249, 249, 249, 0.94));
    backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
    -webkit-backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
    border-top-color: var(--ios-tabbar-border, rgba(0, 0, 0, 0.3));
  }

  /* =================================
     TAB BAR - Android
     ================================= */
  .tabbar.android {
    height: 80px;
    padding-bottom: var(--safe-area-inset-bottom, 0px);
    box-shadow: 0 -1px 3px rgba(0, 0, 0, 0.1);
    border-top: none;
  }

  /* =================================
     TAB ITEM
     ================================= */
  .tabbar-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    min-width: 64px;
    min-height: 44px;
    padding: 4px 8px;
    color: var(--ios-gray, #8E8E93);
    text-decoration: none;
    background: none;
    border: none;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
    transition: color 0.15s ease;
  }

  .tabbar-item.active {
    color: var(--ios-blue, #007AFF);
  }

  /* Touch feedback */
  .tabbar-item:active {
    opacity: 0.6;
    transition: opacity 0.05s ease;
  }

  .tabbar-icon {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tabbar-label {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-tabbar-label-size, 10px);
    font-weight: var(--ios-tabbar-label-weight, 500);
    margin-top: 2px;
  }

  /* Android indicator pill */
  :global(.platform-android) .tabbar-item.active::before {
    content: '';
    position: absolute;
    top: 2px;
    width: 64px;
    height: 32px;
    background: rgba(103, 80, 164, 0.2);
    border-radius: 16px;
    z-index: -1;
  }

  /* =================================
     ACTION SHEET BACKDROP
     ================================= */
  .action-sheet-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 200;
    border: none;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
  }

  /* =================================
     ACTION SHEET
     ================================= */
  .action-sheet {
    position: fixed;
    bottom: 0;
    left: var(--ios-sheet-gap, 8px);
    right: var(--ios-sheet-gap, 8px);
    z-index: 201;
    padding-bottom: calc(var(--safe-area-inset-bottom, 34px) + var(--ios-sheet-gap, 8px));
    animation: slideUp 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.1);
  }

  @keyframes slideUp {
    from {
      transform: translateY(100%);
      opacity: 0.8;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .action-sheet-content {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-sheet-radius, 14px);
    overflow: hidden;
    margin-bottom: var(--ios-sheet-gap, 8px);
  }

  .action-sheet.ios .action-sheet-content {
    background: var(--ios-tabbar-bg, rgba(249, 249, 249, 0.94));
    backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
    -webkit-backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
  }

  .action-sheet-header {
    padding: 16px;
    text-align: center;
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-footnote-size, 13px);
    font-weight: 600;
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 0.5px solid var(--ios-separator, rgba(60, 60, 67, 0.29));
  }

  .action-sheet-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    padding: 16px;
    gap: 12px;
  }

  /* =================================
     ACTION ITEM
     ================================= */
  .action-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px 8px;
    background: none;
    border: none;
    border-radius: var(--ios-button-radius, 10px);
    color: var(--ios-label-primary, #000000);
    text-decoration: none;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
    transition: background 0.15s ease;
  }

  .action-item:active {
    background: var(--ios-fill-tertiary, rgba(120, 120, 128, 0.12));
  }

  .action-icon {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--ios-fill-secondary, rgba(120, 120, 128, 0.16));
    border-radius: 10px;
    color: var(--ios-label-primary, #000000);
    transition: background 0.15s ease, color 0.15s ease;
  }

  .action-item.active .action-icon {
    background: var(--ios-blue, #007AFF);
    color: white;
  }

  .action-label {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-caption1-size, 12px);
    font-weight: 500;
    color: var(--ios-label-primary, #000000);
    text-align: center;
  }

  /* =================================
     CANCEL BUTTON
     ================================= */
  .cancel-button {
    width: 100%;
    height: var(--ios-sheet-action-height, 57px);
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border: none;
    border-radius: var(--ios-sheet-radius, 14px);
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-body-size, 17px);
    font-weight: 600;
    color: var(--ios-blue, #007AFF);
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
    transition: background 0.15s ease;
  }

  .action-sheet.ios .cancel-button {
    background: var(--ios-tabbar-bg, rgba(249, 249, 249, 0.94));
    backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
    -webkit-backdrop-filter: blur(var(--ios-blur-amount, 20px)) saturate(180%);
  }

  .cancel-button:active {
    background: var(--ios-fill-secondary, rgba(120, 120, 128, 0.16));
  }
</style>
