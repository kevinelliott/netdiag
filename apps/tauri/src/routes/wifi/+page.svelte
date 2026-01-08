<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface WifiConnectionInfo {
    interface: string;
    ssid?: string;
    bssid?: string;
    rssi?: number;
    noise?: number;
    snr?: number;
    channel?: number;
    band?: string;
    security?: string;
    tx_rate?: number;
    wifi_standard?: string;
    signal_quality: string;
  }

  let connection = $state<WifiConnectionInfo | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function loadWifiInfo() {
    loading = true;
    error = null;
    try {
      connection = await invoke<WifiConnectionInfo | null>('get_wifi_connection');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadWifiInfo();
  });

  function getSignalPercent(rssi: number): number {
    // Convert RSSI (-100 to -30) to percentage (0-100)
    return Math.max(0, Math.min(100, ((rssi + 100) / 70) * 100));
  }

  function getSignalColor(rssi: number): string {
    if (rssi >= -50) return 'excellent';
    if (rssi >= -60) return 'good';
    if (rssi >= -70) return 'fair';
    if (rssi >= -80) return 'weak';
    return 'very-weak';
  }

  function getSnrQuality(snr: number): { label: string; class: string } {
    if (snr >= 40) return { label: 'Excellent', class: 'excellent' };
    if (snr >= 25) return { label: 'Good', class: 'good' };
    if (snr >= 15) return { label: 'Fair', class: 'fair' };
    if (snr >= 10) return { label: 'Poor', class: 'weak' };
    return { label: 'Very Poor', class: 'very-weak' };
  }
</script>

<div class="page">
  <div class="header-row">
    <h1 class="title">WiFi Status</h1>
    <button class="btn btn-secondary" onclick={loadWifiInfo}>
      {loading ? 'Refreshing...' : 'Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if loading && !connection}
    <p class="loading-text">Loading WiFi information...</p>
  {:else if !connection}
    <div class="card">
      <p class="no-data">No WiFi interface found on this system</p>
    </div>
  {:else if !connection.ssid}
    <div class="card">
      <div class="wifi-status disconnected">
        <span class="status-icon">📶</span>
        <span class="status-text">{connection.signal_quality}</span>
      </div>
      <p class="interface-name">Interface: {connection.interface}</p>
    </div>
  {:else}
    <div class="wifi-grid">
      <!-- Connection Card -->
      <div class="card connection-card">
        <h2 class="card-title">Connection</h2>
        <div class="ssid-row">
          <span class="ssid">{connection.ssid}</span>
          <span class="quality-badge {getSignalColor(connection.rssi!)}">{connection.signal_quality}</span>
        </div>

        <dl class="info-list">
          <dt>Interface</dt>
          <dd>{connection.interface}</dd>
          <dt>BSSID</dt>
          <dd class="mono">{connection.bssid}</dd>
          <dt>Security</dt>
          <dd>{connection.security?.replace('Wpa', 'WPA').replace('Psk', '-PSK') ?? '-'}</dd>
          <dt>WiFi Standard</dt>
          <dd>{connection.wifi_standard?.replace('Wifi', 'WiFi ') ?? '-'}</dd>
        </dl>
      </div>

      <!-- Signal Card -->
      <div class="card signal-card">
        <h2 class="card-title">Signal Strength</h2>

        <div class="signal-gauge">
          <div class="gauge-bar">
            <div
              class="gauge-fill {getSignalColor(connection.rssi!)}"
              style="width: {getSignalPercent(connection.rssi!)}%"
            ></div>
          </div>
          <div class="gauge-value">{connection.rssi} dBm</div>
        </div>

        <dl class="info-list">
          <dt>RSSI</dt>
          <dd class:excellent={connection.rssi! >= -50} class:good={connection.rssi! >= -60 && connection.rssi! < -50} class:fair={connection.rssi! >= -70 && connection.rssi! < -60} class:weak={connection.rssi! < -70}>
            {connection.rssi} dBm
          </dd>
          {#if connection.noise}
            <dt>Noise</dt>
            <dd>{connection.noise} dBm</dd>
          {/if}
          {#if connection.snr}
            <dt>SNR</dt>
            <dd class="{getSnrQuality(connection.snr).class}">{connection.snr} dB ({getSnrQuality(connection.snr).label})</dd>
          {/if}
        </dl>

        <div class="signal-guide">
          <h3 class="guide-title">Signal Guide</h3>
          <ul class="guide-list">
            <li><span class="dot excellent"></span> Excellent: -50 dBm or higher</li>
            <li><span class="dot good"></span> Good: -60 to -50 dBm</li>
            <li><span class="dot fair"></span> Fair: -70 to -60 dBm</li>
            <li><span class="dot weak"></span> Weak: below -70 dBm</li>
          </ul>
        </div>
      </div>

      <!-- Channel Card -->
      <div class="card channel-card">
        <h2 class="card-title">Channel Info</h2>
        <dl class="info-list">
          <dt>Channel</dt>
          <dd class="channel-num">{connection.channel}</dd>
          <dt>Band</dt>
          <dd>{connection.band?.replace('Band', '').replace('_', '.') ?? '-'}</dd>
          <dt>TX Rate</dt>
          <dd>{connection.tx_rate ? `${connection.tx_rate.toFixed(0)} Mbps` : '-'}</dd>
        </dl>
      </div>
    </div>
  {/if}
</div>

<style>
  .page {
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

  .loading-text, .no-data {
    color: var(--text-secondary);
    font-style: italic;
  }

  .wifi-status {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 1.25rem;
  }

  .wifi-status.disconnected {
    color: var(--text-secondary);
  }

  .status-icon {
    font-size: 2rem;
    opacity: 0.5;
  }

  .interface-name {
    margin-top: 1rem;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .wifi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
  }

  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .ssid-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .ssid {
    font-size: 1.5rem;
    font-weight: 600;
  }

  .quality-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .quality-badge.excellent {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .quality-badge.good {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--success);
  }

  .quality-badge.fair {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .quality-badge.weak, .quality-badge.very-weak {
    background-color: rgba(239, 68, 68, 0.15);
    color: var(--error);
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

  .mono {
    font-family: ui-monospace, monospace;
    font-size: 0.875rem;
  }

  .signal-gauge {
    margin-bottom: 1.5rem;
  }

  .gauge-bar {
    height: 12px;
    background-color: var(--bg-tertiary);
    border-radius: 6px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .gauge-fill {
    height: 100%;
    border-radius: 6px;
    transition: width 0.3s ease;
  }

  .gauge-fill.excellent {
    background-color: var(--success);
  }

  .gauge-fill.good {
    background-color: #22c55e;
  }

  .gauge-fill.fair {
    background-color: var(--warning);
  }

  .gauge-fill.weak, .gauge-fill.very-weak {
    background-color: var(--error);
  }

  .gauge-value {
    text-align: center;
    font-size: 1.25rem;
    font-weight: 600;
    font-family: ui-monospace, monospace;
  }

  .excellent {
    color: var(--success);
  }

  .good {
    color: #22c55e;
  }

  .fair {
    color: var(--warning);
  }

  .weak, .very-weak {
    color: var(--error);
  }

  .signal-guide {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .guide-title {
    margin: 0 0 0.75rem 0;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .guide-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .guide-list li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot.excellent {
    background-color: var(--success);
  }

  .dot.good {
    background-color: #22c55e;
  }

  .dot.fair {
    background-color: var(--warning);
  }

  .dot.weak {
    background-color: var(--error);
  }

  .channel-num {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent);
  }
</style>
