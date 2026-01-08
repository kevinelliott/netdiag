<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

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
      // Load data in parallel
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

      // Check connectivity
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
</script>

<div class="dashboard">
  <div class="header-row">
    <h1 class="title">Network Dashboard</h1>
    <button class="btn btn-secondary" onclick={() => loadDashboardData()}>
      {loading ? 'Refreshing...' : 'Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <span>Error: {error}</span>
    </div>
  {/if}

  <div class="grid">
    <!-- System Info Card -->
    <div class="card">
      <h2 class="card-title">System Information</h2>
      {#if systemInfo}
        <dl class="info-list">
          <dt>Hostname</dt>
          <dd>{systemInfo.hostname}</dd>
          <dt>OS</dt>
          <dd>{systemInfo.os_type} {systemInfo.os_version}</dd>
          <dt>Architecture</dt>
          <dd>{systemInfo.architecture}</dd>
          {#if systemInfo.uptime_seconds}
            <dt>Uptime</dt>
            <dd>{formatUptime(systemInfo.uptime_seconds)}</dd>
          {/if}
        </dl>
      {:else if loading}
        <p class="loading-text">Loading...</p>
      {/if}
    </div>

    <!-- Connectivity Status Card -->
    <div class="card">
      <h2 class="card-title">Connectivity Status</h2>
      <div class="connectivity-status">
        {#if connectivityStatus === 'checking'}
          <span class="status-icon checking">⏳</span>
          <span>Checking connectivity...</span>
        {:else if connectivityStatus === 'ok'}
          <span class="status-icon ok">✓</span>
          <span class="status-ok">Connected</span>
        {:else}
          <span class="status-icon error">✗</span>
          <span class="status-error">No connectivity</span>
        {/if}
      </div>
      {#if connectivityResult && !connectivityResult.error}
        <dl class="info-list mini">
          <dt>Latency</dt>
          <dd>{connectivityResult.avg_ms?.toFixed(1) ?? '-'} ms</dd>
          <dt>Packet Loss</dt>
          <dd class:status-error={connectivityResult.loss_percent > 0}>
            {connectivityResult.loss_percent.toFixed(1)}%
          </dd>
        </dl>
      {/if}
    </div>

    <!-- Network Overview Card -->
    <div class="card">
      <h2 class="card-title">Network Overview</h2>
      <dl class="info-list">
        <dt>Default Gateway</dt>
        <dd>{defaultGateway ?? 'Not available'}</dd>
        <dt>DNS Servers</dt>
        <dd>{dnsServers.length > 0 ? dnsServers.join(', ') : 'Not available'}</dd>
        <dt>Active Interfaces</dt>
        <dd>{interfaces.filter(i => i.is_up && !i.is_loopback).length}</dd>
      </dl>
    </div>

    <!-- Default Interface Card -->
    <div class="card">
      <h2 class="card-title">Default Interface</h2>
      {#if interfaces.find(i => i.is_default)}
        {@const iface = interfaces.find(i => i.is_default)!}
        <dl class="info-list">
          <dt>Name</dt>
          <dd>{iface.friendly_name || iface.name}</dd>
          <dt>Type</dt>
          <dd>{iface.interface_type}</dd>
          <dt>IPv4</dt>
          <dd>{iface.ipv4_addresses[0] ?? 'Not assigned'}</dd>
          {#if iface.mac_address}
            <dt>MAC</dt>
            <dd class="mono">{iface.mac_address}</dd>
          {/if}
        </dl>
      {:else if loading}
        <p class="loading-text">Loading...</p>
      {:else}
        <p class="no-data">No default interface found</p>
      {/if}
    </div>
  </div>

  <!-- Quick Actions -->
  <div class="quick-actions">
    <h2 class="section-title">Quick Actions</h2>
    <div class="action-grid">
      <a href="/ping" class="action-card">
        <span class="action-icon">📡</span>
        <span class="action-label">Ping Test</span>
      </a>
      <a href="/traceroute" class="action-card">
        <span class="action-icon">🛤️</span>
        <span class="action-label">Traceroute</span>
      </a>
      <a href="/dns" class="action-card">
        <span class="action-icon">🔍</span>
        <span class="action-label">DNS Lookup</span>
      </a>
      <a href="/interfaces" class="action-card">
        <span class="action-icon">🔌</span>
        <span class="action-label">Interfaces</span>
      </a>
    </div>
  </div>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title {
    margin: 0;
    font-size: 1.5rem;
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    color: var(--error);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
  }

  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .info-list {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.5rem 1rem;
    margin: 0;
  }

  .info-list dt {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .info-list dd {
    margin: 0;
    font-weight: 500;
  }

  .info-list.mini {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .loading-text, .no-data {
    color: var(--text-secondary);
    font-style: italic;
  }

  .mono {
    font-family: ui-monospace, monospace;
    font-size: 0.875rem;
  }

  .connectivity-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 1.125rem;
    font-weight: 500;
  }

  .status-icon {
    font-size: 1.5rem;
  }

  .status-icon.ok {
    color: var(--success);
  }

  .status-icon.error {
    color: var(--error);
  }

  .status-icon.checking {
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .section-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }

  .action-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.25rem 1rem;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    text-decoration: none;
    color: var(--text-primary);
    transition: all 0.15s ease;
  }

  .action-card:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .action-icon {
    font-size: 2rem;
  }

  .action-label {
    font-weight: 500;
  }
</style>
