<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getPlatform } from '$lib/platform';
  import { Icon } from '$lib/components';

  const platform = getPlatform();

  interface SystemInfo {
    hostname: string;
    os_type: string;
    os_version: string;
    architecture: string;
    uptime_seconds?: number;
  }

  interface InterfaceInfo {
    name: string;
    friendly_name?: string;
    mac_address?: string;
    ipv4_addresses: string[];
    ipv6_addresses: string[];
    is_up: boolean;
    is_loopback: boolean;
    is_default: boolean;
    interface_type: string;
  }

  interface PingResult {
    target: string;
    resolved_ip?: string;
    sent: number;
    received: number;
    lost: number;
    loss_percent: number;
    min_ms?: number;
    avg_ms?: number;
    max_ms?: number;
    jitter_ms?: number;
    error?: string;
  }

  let systemInfo = $state<SystemInfo | null>(null);
  let interfaces = $state<InterfaceInfo[]>([]);
  let defaultGateway = $state<string | null>(null);
  let dnsServers = $state<string[]>([]);
  let connectivityStatus = $state<'checking' | 'ok' | 'error'>('checking');
  let connectivityResult = $state<PingResult | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h ${mins}m`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  }

  async function loadDashboardData() {
    loading = true;
    error = null;

    try {
      const [sysInfo, ifaces, gateway, dns] = await Promise.all([
        invoke<SystemInfo>('get_system_info'),
        invoke<InterfaceInfo[]>('get_interfaces'),
        invoke<string | null>('get_default_gateway'),
        invoke<string[]>('get_dns_servers'),
      ]);

      systemInfo = sysInfo;
      interfaces = ifaces;
      defaultGateway = gateway;
      dnsServers = dns;

      connectivityStatus = 'checking';
      const result = await invoke<PingResult>('check_connectivity', { target: '8.8.8.8' });
      connectivityResult = result;
      connectivityStatus = result.error || result.received === 0 ? 'error' : 'ok';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      connectivityStatus = 'error';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDashboardData();
  });

  const quickActions = [
    { href: '/ping', icon: 'ping', label: 'Ping Test' },
    { href: '/traceroute', icon: 'traceroute', label: 'Traceroute' },
    { href: '/dns', icon: 'dns', label: 'DNS Lookup' },
    { href: '/wifi', icon: 'wifi', label: 'WiFi Status' },
    { href: '/interfaces', icon: 'interfaces', label: 'Interfaces' },
  ] as const;
</script>

<div class="page">
  <!-- Header -->
  <header class="page-header">
    <h1 class="page-title">Dashboard</h1>
    <button
      class="refresh-btn"
      onclick={() => loadDashboardData()}
      disabled={loading}
      aria-label="Refresh"
    >
      <Icon name="refresh" size={22} />
    </button>
  </header>

  {#if error}
    <div class="error-banner">
      <Icon name="warning" size={20} />
      <span>{error}</span>
    </div>
  {/if}

  <!-- Status Section -->
  <section class="section">
    <div class="section-header">Status</div>
    <div class="section-content">
      <div class="status-row">
        <div class="status-label">Internet</div>
        <div class="status-value">
          {#if connectivityStatus === 'checking'}
            <span class="status-checking">Checking...</span>
          {:else if connectivityStatus === 'ok'}
            <span class="status-ok">Connected</span>
          {:else}
            <span class="status-error">Offline</span>
          {/if}
        </div>
      </div>
      {#if connectivityResult && !connectivityResult.error}
        <div class="status-row">
          <div class="status-label">Latency</div>
          <div class="status-value">{connectivityResult.avg_ms?.toFixed(1) ?? '-'} ms</div>
        </div>
      {/if}
      <div class="status-row">
        <div class="status-label">Gateway</div>
        <div class="status-value">{defaultGateway ?? 'Unknown'}</div>
      </div>
      <div class="status-row last">
        <div class="status-label">Active Interfaces</div>
        <div class="status-value">{interfaces.filter(i => i.is_up && !i.is_loopback).length}</div>
      </div>
    </div>
  </section>

  <!-- System Info Section -->
  {#if systemInfo}
    <section class="section">
      <div class="section-header">System</div>
      <div class="section-content">
        <div class="status-row">
          <div class="status-label">Hostname</div>
          <div class="status-value">{systemInfo.hostname}</div>
        </div>
        <div class="status-row">
          <div class="status-label">OS</div>
          <div class="status-value">{systemInfo.os_type}</div>
        </div>
        <div class="status-row">
          <div class="status-label">Version</div>
          <div class="status-value">{systemInfo.os_version}</div>
        </div>
        {#if systemInfo.uptime_seconds}
          <div class="status-row last">
            <div class="status-label">Uptime</div>
            <div class="status-value">{formatUptime(systemInfo.uptime_seconds)}</div>
          </div>
        {:else}
          <div class="status-row last">
            <div class="status-label">Architecture</div>
            <div class="status-value">{systemInfo.architecture}</div>
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <!-- Default Interface Section -->
  {#if interfaces.find(i => i.is_default)}
    {@const iface = interfaces.find(i => i.is_default)!}
    <section class="section">
      <div class="section-header">Primary Interface</div>
      <div class="section-content">
        <div class="status-row">
          <div class="status-label">Name</div>
          <div class="status-value">{iface.friendly_name || iface.name}</div>
        </div>
        <div class="status-row">
          <div class="status-label">Type</div>
          <div class="status-value">{iface.interface_type}</div>
        </div>
        <div class="status-row">
          <div class="status-label">IPv4</div>
          <div class="status-value mono">{iface.ipv4_addresses[0] ?? 'Not assigned'}</div>
        </div>
        {#if iface.mac_address}
          <div class="status-row last">
            <div class="status-label">MAC</div>
            <div class="status-value mono">{iface.mac_address}</div>
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <!-- Quick Actions Section -->
  <section class="section">
    <div class="section-header">Quick Actions</div>
    <div class="action-grid">
      {#each quickActions as action}
        <a href={action.href} class="action-card">
          <div class="action-icon">
            <Icon name={action.icon} size={28} />
          </div>
          <span class="action-label">{action.label}</span>
        </a>
      {/each}
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-bottom: 24px;
  }

  /* Header */
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding: 8px 16px 16px 16px;
  }

  .page-title {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-large-title-size, 34px);
    font-weight: var(--ios-large-title-weight, 700);
    letter-spacing: var(--ios-large-title-tracking, 0.37px);
    line-height: var(--ios-large-title-leading, 41px);
    color: var(--ios-label-primary, var(--text-primary));
    margin: 0;
  }

  .refresh-btn {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--ios-blue, var(--accent));
    cursor: pointer;
    border-radius: 22px;
    -webkit-tap-highlight-color: transparent;
  }

  .refresh-btn:active {
    background: var(--ios-fill-tertiary, rgba(120, 120, 128, 0.12));
  }

  .refresh-btn:disabled {
    opacity: 0.5;
  }

  /* Error Banner */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 16px 16px 16px;
    padding: 12px 16px;
    background: rgba(255, 59, 48, 0.1);
    border-radius: var(--ios-grouped-radius, 10px);
    color: var(--ios-red, #FF3B30);
    font-size: var(--ios-body-size, 17px);
  }

  /* Sections */
  .section {
    margin-bottom: 35px;
  }

  .section-header {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-footnote-size, 13px);
    font-weight: 400;
    text-transform: uppercase;
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
    padding: 0 16px 8px 16px;
    letter-spacing: -0.08px;
  }

  .section-content {
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-grouped-radius, 10px);
    margin: 0 16px;
    overflow: hidden;
  }

  /* Status Rows */
  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: var(--ios-cell-min-height, 44px);
    padding: var(--ios-cell-padding-vertical, 11px) var(--ios-cell-padding-horizontal, 16px);
    position: relative;
  }

  .status-row::after {
    content: '';
    position: absolute;
    left: var(--ios-cell-padding-horizontal, 16px);
    right: 0;
    bottom: 0;
    height: 0.5px;
    background: var(--ios-separator, rgba(60, 60, 67, 0.29));
  }

  .status-row.last::after {
    display: none;
  }

  .status-label {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-body-size, 17px);
    color: var(--ios-label-primary, #000000);
  }

  .status-value {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-body-size, 17px);
    color: var(--ios-label-secondary, rgba(60, 60, 67, 0.6));
  }

  .status-value.mono {
    font-family: ui-monospace, 'SF Mono', monospace;
    font-size: 15px;
  }

  .status-ok {
    color: var(--ios-green, #34C759);
  }

  .status-error {
    color: var(--ios-red, #FF3B30);
  }

  .status-checking {
    color: var(--ios-label-tertiary, rgba(60, 60, 67, 0.3));
  }

  /* Quick Actions Grid */
  .action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 12px;
    padding: 0 16px;
  }

  .action-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px 8px;
    background: var(--ios-bg-grouped-secondary, #FFFFFF);
    border-radius: var(--ios-grouped-radius, 10px);
    text-decoration: none;
    color: var(--ios-label-primary, #000000);
    -webkit-tap-highlight-color: transparent;
    -webkit-user-select: none;
    user-select: none;
    transition: background 0.15s ease;
  }

  .action-card:active {
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
    color: var(--ios-blue, #007AFF);
  }

  .action-label {
    font-family: var(--ios-font-family, -apple-system, BlinkMacSystemFont, system-ui, sans-serif);
    font-size: var(--ios-caption1-size, 12px);
    font-weight: 500;
    text-align: center;
    color: var(--ios-label-primary, #000000);
  }

  /* Dark mode adjustments */
  @media (prefers-color-scheme: dark) {
    .error-banner {
      background: rgba(255, 69, 58, 0.15);
    }
  }
</style>
