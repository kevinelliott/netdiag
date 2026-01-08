<script lang="ts">
  import '../app.css';

  let { children } = $props();
</script>

<svelte:head>
  <title>NetDiag - Network Diagnostics</title>
</svelte:head>

<div class="app">
  <header class="header">
    <nav class="nav">
      <a href="/" class="nav-brand">
        <span class="brand-icon">🌐</span>
        <span class="brand-text">NetDiag</span>
      </a>
      <div class="nav-links">
        <a href="/" class="nav-link">Dashboard</a>
        <a href="/interfaces" class="nav-link">Interfaces</a>
        <a href="/ping" class="nav-link">Ping</a>
        <a href="/traceroute" class="nav-link">Traceroute</a>
        <a href="/dns" class="nav-link">DNS</a>
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
</style>
