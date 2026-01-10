<script lang="ts">
  import { getPlatform } from '$lib/platform';
  import Sidebar from '../navigation/Sidebar.svelte';
  import TabBar from '../navigation/TabBar.svelte';

  let { children } = $props();
  const platform = getPlatform();
</script>

<div
  class="app-shell"
  class:desktop={platform?.isDesktop}
  class:mobile={platform?.isMobile}
>
  {#if platform?.isDesktop}
    <Sidebar />
    <main class="main-content desktop">
      <div class="content-wrapper">
        {@render children()}
      </div>
    </main>
  {:else}
    <main class="main-content mobile">
      <div class="content-wrapper">
        {@render children()}
      </div>
    </main>
    <TabBar />
  {/if}
</div>

<style>
  .app-shell {
    min-height: 100vh;
    background: var(--bg-primary);
  }

  .main-content {
    min-height: 100vh;
  }

  .main-content.desktop {
    margin-left: var(--sidebar-width, 240px);
  }

  .main-content.mobile {
    padding-bottom: var(--tabbar-height, 56px);
  }

  .content-wrapper {
    padding: var(--space-lg, 24px);
    max-width: 1200px;
    margin: 0 auto;
  }

  /* Mobile adjustments */
  :global(.device-mobile) .content-wrapper {
    padding: var(--space-md, 16px);
    padding-bottom: calc(var(--tabbar-height, 56px) + var(--space-md, 16px));
  }

  /* iOS safe area */
  :global(.platform-ios) .content-wrapper {
    padding-top: calc(var(--safe-area-inset-top, 0px) + var(--space-md, 16px));
    padding-bottom: calc(var(--tabbar-height, 83px) + var(--space-md, 16px));
  }

  /* macOS title bar space */
  :global(.platform-macos) .main-content.desktop {
    padding-top: var(--titlebar-height, 28px);
  }

  /* Windows title bar space */
  :global(.platform-windows) .main-content.desktop {
    padding-top: var(--titlebar-height, 32px);
  }
</style>
