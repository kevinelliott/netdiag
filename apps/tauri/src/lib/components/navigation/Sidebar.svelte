<script lang="ts">
  import { page } from '$app/stores';
  import { getPlatform } from '$lib/platform';

  const platform = getPlatform();

  const sections = [
    {
      label: 'Overview',
      items: [
        { href: '/', label: 'Dashboard', icon: 'dashboard' },
        { href: '/diagnose', label: 'Diagnose', icon: 'diagnose' },
      ],
    },
    {
      label: 'Network',
      items: [
        { href: '/interfaces', label: 'Interfaces', icon: 'interfaces' },
        { href: '/wifi', label: 'WiFi', icon: 'wifi' },
        { href: '/speed', label: 'Speed', icon: 'speed' },
      ],
    },
    {
      label: 'Tools',
      items: [
        { href: '/ping', label: 'Ping', icon: 'ping' },
        { href: '/traceroute', label: 'Traceroute', icon: 'traceroute' },
        { href: '/dns', label: 'DNS', icon: 'dns' },
        { href: '/capture', label: 'Capture', icon: 'capture' },
      ],
    },
    {
      label: 'System',
      items: [
        { href: '/report', label: 'Report', icon: 'report' },
        { href: '/fix', label: 'Fix', icon: 'fix' },
      ],
    },
  ];

  function getIcon(icon: string): string {
    const icons: Record<string, string> = {
      dashboard: '⊞',
      diagnose: '⚕',
      interfaces: '⎔',
      wifi: '◠',
      speed: '⚡',
      ping: '◎',
      traceroute: '⤳',
      dns: '⌕',
      capture: '◉',
      report: '📄',
      fix: '🔧',
    };
    return icons[icon] || '•';
  }
</script>

<aside class="sidebar" class:vibrancy={platform?.supportsVibrancy}>
  <div class="sidebar-header">
    <div class="brand">
      <span class="brand-icon">⬡</span>
      <span class="brand-text">NetDiag</span>
    </div>
  </div>

  <nav class="sidebar-nav">
    {#each sections as section}
      <div class="nav-section">
        <span class="section-label">{section.label}</span>
        {#each section.items as item}
          <a
            href={item.href}
            class="nav-item"
            class:active={$page.url.pathname === item.href}
          >
            <span class="nav-icon">{getIcon(item.icon)}</span>
            <span class="nav-label">{item.label}</span>
          </a>
        {/each}
      </div>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <span class="version">v0.1.0</span>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width, 240px);
    height: 100vh;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    position: fixed;
    left: 0;
    top: 0;
    z-index: 100;
  }

  .sidebar.vibrancy {
    background: var(--vibrancy-bg, rgba(246, 246, 246, 0.75));
    backdrop-filter: blur(var(--blur-amount, 20px)) saturate(180%);
    -webkit-backdrop-filter: blur(var(--blur-amount, 20px)) saturate(180%);
    border-right-color: var(--vibrancy-border, rgba(0, 0, 0, 0.1));
  }

  .sidebar-header {
    padding: var(--space-md, 16px);
    padding-top: calc(var(--titlebar-height, 0px) + var(--space-md, 16px));
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 8px);
  }

  .brand-icon {
    font-size: 24px;
    color: var(--accent);
  }

  .brand-text {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm, 8px);
  }

  .nav-section {
    margin-bottom: var(--space-md, 16px);
  }

  .section-label {
    display: block;
    font-size: var(--font-size-section-label, 11px);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    padding: var(--space-sm, 8px) var(--space-md, 16px) var(--space-xs, 4px);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm, 8px);
    padding: var(--space-sm, 8px) var(--space-md, 16px);
    margin: 2px var(--space-xs, 4px);
    border-radius: var(--radius-md, 6px);
    color: var(--text-primary);
    text-decoration: none;
    font-size: var(--font-size-sidebar, 13px);
    transition: background var(--transition-fast, 0.1s ease);
    border: 1px solid transparent;
  }

  .nav-item:hover {
    background: var(--bg-tertiary);
  }

  .nav-item.active {
    background: var(--accent);
    color: white;
  }

  .nav-icon {
    width: 20px;
    text-align: center;
    font-size: 14px;
    opacity: 0.8;
  }

  .nav-item.active .nav-icon {
    opacity: 1;
  }

  .nav-label {
    font-weight: 500;
  }

  .sidebar-footer {
    padding: var(--space-md, 16px);
    border-top: 1px solid var(--border);
  }

  .version {
    font-size: 11px;
    color: var(--text-secondary);
  }

  /* Platform-specific adjustments */
  :global(.platform-macos) .sidebar {
    padding-top: 28px;
  }

  :global(.platform-windows) .sidebar {
    padding-top: 32px;
  }
</style>
