<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';

  let { children } = $props();
  let menuOpen = $state(false);

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  function closeMenu() {
    menuOpen = false;
  }

  // Close menu when route changes
  $effect(() => {
    $page.url;
    menuOpen = false;
  });
</script>

<svelte:head>
  <title>NetDiag - Network Diagnostics</title>
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
</svelte:head>

<div class="app">
  <header class="header">
    <nav class="nav">
      <a href="/" class="nav-brand">
        <span class="brand-icon">🌐</span>
        <span class="brand-text">NetDiag</span>
      </a>

      <!-- Mobile menu button -->
      <button class="menu-toggle" onclick={toggleMenu} aria-label="Toggle menu">
        <span class="menu-icon" class:open={menuOpen}></span>
      </button>

      <!-- Navigation links -->
      <div class="nav-links" class:open={menuOpen}>
        <a href="/" class="nav-link" class:active={$page.url.pathname === '/'} onclick={closeMenu}>Dashboard</a>
        <a href="/diagnose" class="nav-link" class:active={$page.url.pathname === '/diagnose'} onclick={closeMenu}>Diagnose</a>
        <a href="/interfaces" class="nav-link" class:active={$page.url.pathname === '/interfaces'} onclick={closeMenu}>Interfaces</a>
        <a href="/wifi" class="nav-link" class:active={$page.url.pathname === '/wifi'} onclick={closeMenu}>WiFi</a>
        <a href="/speed" class="nav-link" class:active={$page.url.pathname === '/speed'} onclick={closeMenu}>Speed</a>
        <a href="/ping" class="nav-link" class:active={$page.url.pathname === '/ping'} onclick={closeMenu}>Ping</a>
        <a href="/traceroute" class="nav-link" class:active={$page.url.pathname === '/traceroute'} onclick={closeMenu}>Traceroute</a>
        <a href="/dns" class="nav-link" class:active={$page.url.pathname === '/dns'} onclick={closeMenu}>DNS</a>
        <a href="/report" class="nav-link" class:active={$page.url.pathname === '/report'} onclick={closeMenu}>Report</a>
        <a href="/fix" class="nav-link" class:active={$page.url.pathname === '/fix'} onclick={closeMenu}>Fix</a>
        <a href="/capture" class="nav-link" class:active={$page.url.pathname === '/capture'} onclick={closeMenu}>Capture</a>
      </div>
    </nav>
  </header>

  <main class="main">
    {@render children()}
  </main>

  <footer class="footer">
    <span>NetDiag v0.1.0</span>
  </footer>
</div>

<!-- Mobile menu overlay -->
{#if menuOpen}
  <button class="menu-overlay" onclick={closeMenu} aria-label="Close menu"></button>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .header {
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    padding: 0 1rem;
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1400px;
    margin: 0 auto;
    height: 56px;
  }

  .nav-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    text-decoration: none;
    color: var(--text-primary);
    font-weight: 600;
    font-size: 1.25rem;
  }

  .brand-icon {
    font-size: 1.5rem;
  }

  /* Mobile menu toggle button */
  .menu-toggle {
    display: none;
    width: 44px;
    height: 44px;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    justify-content: center;
    align-items: center;
  }

  .menu-icon {
    display: block;
    width: 24px;
    height: 2px;
    background-color: var(--text-primary);
    position: relative;
    transition: background-color 0.2s ease;
  }

  .menu-icon::before,
  .menu-icon::after {
    content: '';
    position: absolute;
    width: 24px;
    height: 2px;
    background-color: var(--text-primary);
    left: 0;
    transition: transform 0.2s ease;
  }

  .menu-icon::before {
    top: -7px;
  }

  .menu-icon::after {
    top: 7px;
  }

  .menu-icon.open {
    background-color: transparent;
  }

  .menu-icon.open::before {
    transform: translateY(7px) rotate(45deg);
  }

  .menu-icon.open::after {
    transform: translateY(-7px) rotate(-45deg);
  }

  .nav-links {
    display: flex;
    gap: 0.5rem;
  }

  .nav-link {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    text-decoration: none;
    color: var(--text-secondary);
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .nav-link:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-link.active {
    background-color: var(--accent);
    color: white;
  }

  .main {
    flex: 1;
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;
  }

  .footer {
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    padding: 0.75rem 1rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  /* Menu overlay for mobile */
  .menu-overlay {
    display: none;
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 90;
    border: none;
    cursor: pointer;
  }

  /* Mobile styles */
  @media (max-width: 768px) {
    .menu-toggle {
      display: flex;
    }

    .nav-links {
      position: fixed;
      top: 56px;
      left: 0;
      right: 0;
      flex-direction: column;
      background-color: var(--bg-secondary);
      border-bottom: 1px solid var(--border);
      padding: 0.5rem;
      gap: 0.25rem;
      transform: translateY(-100%);
      opacity: 0;
      visibility: hidden;
      transition: transform 0.3s ease, opacity 0.3s ease;
      z-index: 95;
    }

    .nav-links.open {
      transform: translateY(0);
      opacity: 1;
      visibility: visible;
    }

    .nav-link {
      padding: 0.75rem 1rem;
      min-height: 44px;
      display: flex;
      align-items: center;
    }

    .menu-overlay {
      display: block;
    }

    .main {
      padding: 1rem;
    }
  }

  /* Small mobile */
  @media (max-width: 480px) {
    .brand-text {
      display: none;
    }

    .nav-brand {
      font-size: 1.5rem;
    }

    .main {
      padding: 0.75rem;
    }
  }
</style>
