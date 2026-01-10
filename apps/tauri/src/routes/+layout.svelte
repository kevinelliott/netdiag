<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { initializePlatform, isPlatformReady } from '$lib/platform';
  import { AppShell } from '$lib/components';

  let { children } = $props();
  let ready = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      await initializePlatform();
      ready = true;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to initialize platform';
      // Still show the app even if platform detection fails
      ready = true;
    }
  });
</script>

<svelte:head>
  <title>NetDiag - Network Diagnostics</title>
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
  <meta name="theme-color" content="#111827" media="(prefers-color-scheme: dark)" />
  <meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
</svelte:head>

{#if ready}
  <AppShell>
    {@render children()}
  </AppShell>
{:else}
  <div class="loading-screen">
    <div class="loading-content">
      <div class="loading-icon">⬡</div>
      <div class="loading-text">NetDiag</div>
      <div class="loading-spinner"></div>
    </div>
  </div>
{/if}

<style>
  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-primary);
  }

  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .loading-icon {
    font-size: 48px;
    color: var(--accent);
  }

  .loading-text {
    font-size: 24px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .loading-spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
