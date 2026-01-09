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

  interface AccessPointInfo {
    ssid: string;
    bssid: string;
    rssi: number;
    signal_quality: number;
    channel: number;
    band: string;
    security: string;
    wifi_standard: string;
    is_connected: boolean;
  }

  interface ChannelAnalysis {
    channel: number;
    band: string;
    network_count: number;
    interference_level: string;
    is_dfs: boolean;
    is_recommended: boolean;
    is_current: boolean;
  }

  interface InterferenceReport {
    current_channel?: number;
    snr_rating: string;
    channel_utilization?: number;
    overlapping_networks: string[];
    recommendations: string[];
  }

  type Tab = 'status' | 'scan' | 'channels' | 'interference';

  let activeTab = $state<Tab>('status');
  let connection = $state<WifiConnectionInfo | null>(null);
  let accessPoints = $state<AccessPointInfo[]>([]);
  let channels = $state<ChannelAnalysis[]>([]);
  let interference = $state<InterferenceReport | null>(null);
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

  async function scanNetworks() {
    loading = true;
    error = null;
    try {
      accessPoints = await invoke<AccessPointInfo[]>('scan_wifi_networks', { interface: null });
      // Sort by signal strength
      accessPoints.sort((a, b) => b.rssi - a.rssi);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function loadChannels() {
    loading = true;
    error = null;
    try {
      channels = await invoke<ChannelAnalysis[]>('analyze_wifi_channels', { interface: null });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function loadInterference() {
    loading = true;
    error = null;
    try {
      interference = await invoke<InterferenceReport>('check_wifi_interference', { interface: null });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function switchTab(tab: Tab) {
    activeTab = tab;
    error = null;
    if (tab === 'status' && !connection) {
      loadWifiInfo();
    } else if (tab === 'scan' && accessPoints.length === 0) {
      scanNetworks();
    } else if (tab === 'channels' && channels.length === 0) {
      loadChannels();
    } else if (tab === 'interference' && !interference) {
      loadInterference();
    }
  }

  function refresh() {
    if (activeTab === 'status') loadWifiInfo();
    else if (activeTab === 'scan') scanNetworks();
    else if (activeTab === 'channels') loadChannels();
    else if (activeTab === 'interference') loadInterference();
  }

  onMount(() => {
    loadWifiInfo();
  });

  function getSignalPercent(rssi: number): number {
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

  function getInterferenceClass(level: string): string {
    switch (level) {
      case 'Low': return 'excellent';
      case 'Medium': return 'fair';
      case 'High': return 'weak';
      case 'Severe': return 'very-weak';
      default: return '';
    }
  }
</script>

<div class="page">
  <div class="header-row">
    <h1 class="title">WiFi</h1>
    <button class="btn btn-secondary" onclick={refresh}>
      {loading ? 'Loading...' : 'Refresh'}
    </button>
  </div>

  <div class="tabs">
    <button
      class="tab"
      class:active={activeTab === 'status'}
      onclick={() => switchTab('status')}
    >
      Status
    </button>
    <button
      class="tab"
      class:active={activeTab === 'scan'}
      onclick={() => switchTab('scan')}
    >
      Scan
    </button>
    <button
      class="tab"
      class:active={activeTab === 'channels'}
      onclick={() => switchTab('channels')}
    >
      Channels
    </button>
    <button
      class="tab"
      class:active={activeTab === 'interference'}
      onclick={() => switchTab('interference')}
    >
      Interference
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if activeTab === 'status'}
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
            <dd class="{getSignalColor(connection.rssi!)}">{connection.rssi} dBm</dd>
            {#if connection.noise}
              <dt>Noise</dt>
              <dd>{connection.noise} dBm</dd>
            {/if}
            {#if connection.snr}
              <dt>SNR</dt>
              <dd class="{getSnrQuality(connection.snr).class}">{connection.snr} dB ({getSnrQuality(connection.snr).label})</dd>
            {/if}
          </dl>
        </div>

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

  {:else if activeTab === 'scan'}
    {#if loading}
      <p class="loading-text">Scanning for networks...</p>
    {:else if accessPoints.length === 0}
      <div class="card">
        <p class="no-data">No networks found. Try scanning again.</p>
      </div>
    {:else}
      <div class="card">
        <h2 class="card-title">Nearby Networks ({accessPoints.length})</h2>
        <div class="network-list">
          {#each accessPoints as ap}
            <div class="network-item" class:connected={ap.is_connected}>
              <div class="network-info">
                <div class="network-name">
                  <span class="ssid">{ap.ssid || '(Hidden Network)'}</span>
                  {#if ap.is_connected}
                    <span class="connected-badge">Connected</span>
                  {/if}
                </div>
                <div class="network-details">
                  <span>Ch {ap.channel}</span>
                  <span>{ap.band.replace('Band', '').replace('_', '.')}</span>
                  <span>{ap.security.replace('Wpa', 'WPA').replace('Psk', '-PSK')}</span>
                </div>
              </div>
              <div class="network-signal">
                <span class="signal-value {getSignalColor(ap.rssi)}">{ap.rssi} dBm</span>
                <div class="mini-gauge">
                  <div class="mini-gauge-fill {getSignalColor(ap.rssi)}" style="width: {getSignalPercent(ap.rssi)}%"></div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

  {:else if activeTab === 'channels'}
    {#if loading}
      <p class="loading-text">Analyzing channels...</p>
    {:else if channels.length === 0}
      <div class="card">
        <p class="no-data">No channel data available.</p>
      </div>
    {:else}
      <div class="channels-grid">
        <div class="card">
          <h2 class="card-title">2.4 GHz Channels</h2>
          <div class="channel-list">
            {#each channels.filter(c => c.band.includes('2_4')) as ch}
              <div class="channel-item" class:current={ch.is_current} class:recommended={ch.is_recommended}>
                <span class="channel-number">{ch.channel}</span>
                <span class="channel-networks">{ch.network_count} networks</span>
                <span class="channel-interference {getInterferenceClass(ch.interference_level)}">
                  {ch.interference_level}
                </span>
                {#if ch.is_current}
                  <span class="current-badge">Current</span>
                {/if}
                {#if ch.is_recommended}
                  <span class="recommended-badge">Recommended</span>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        <div class="card">
          <h2 class="card-title">5 GHz Channels</h2>
          <div class="channel-list">
            {#each channels.filter(c => c.band.includes('5')) as ch}
              <div class="channel-item" class:current={ch.is_current} class:recommended={ch.is_recommended} class:dfs={ch.is_dfs}>
                <span class="channel-number">{ch.channel}</span>
                <span class="channel-networks">{ch.network_count} networks</span>
                <span class="channel-interference {getInterferenceClass(ch.interference_level)}">
                  {ch.interference_level}
                </span>
                {#if ch.is_dfs}
                  <span class="dfs-badge">DFS</span>
                {/if}
                {#if ch.is_current}
                  <span class="current-badge">Current</span>
                {/if}
                {#if ch.is_recommended}
                  <span class="recommended-badge">Recommended</span>
                {/if}
              </div>
            {/each}
            {#if channels.filter(c => c.band.includes('5')).length === 0}
              <p class="no-data">No 5 GHz channels detected</p>
            {/if}
          </div>
        </div>
      </div>
    {/if}

  {:else if activeTab === 'interference'}
    {#if loading}
      <p class="loading-text">Analyzing interference...</p>
    {:else if !interference}
      <div class="card">
        <p class="no-data">No interference data available.</p>
      </div>
    {:else}
      <div class="interference-grid">
        <div class="card">
          <h2 class="card-title">Signal Quality</h2>
          <dl class="info-list">
            <dt>Current Channel</dt>
            <dd class="channel-num">{interference.current_channel ?? '-'}</dd>
            <dt>SNR Rating</dt>
            <dd class="{getInterferenceClass(interference.snr_rating === 'Excellent' ? 'Low' : interference.snr_rating === 'Good' ? 'Low' : interference.snr_rating === 'Fair' ? 'Medium' : 'High')}">
              {interference.snr_rating}
            </dd>
            {#if interference.channel_utilization !== undefined && interference.channel_utilization !== null}
              <dt>Channel Utilization</dt>
              <dd>{interference.channel_utilization.toFixed(1)}%</dd>
            {/if}
          </dl>
        </div>

        <div class="card">
          <h2 class="card-title">Overlapping Networks</h2>
          {#if interference.overlapping_networks.length === 0}
            <p class="good-message">No significant channel overlap detected</p>
          {:else}
            <ul class="overlap-list">
              {#each interference.overlapping_networks as network}
                <li>{network}</li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="card recommendations-card">
          <h2 class="card-title">Recommendations</h2>
          <ul class="recommendations-list">
            {#each interference.recommendations as rec}
              <li>{rec}</li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
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

  .tabs {
    display: flex;
    gap: 0.25rem;
    background-color: var(--bg-secondary);
    padding: 0.25rem;
    border-radius: 0.5rem;
  }

  .tab {
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-weight: 500;
    cursor: pointer;
    border-radius: 0.375rem;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
  }

  .tab.active {
    background-color: var(--accent);
    color: white;
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

  .wifi-grid, .channels-grid, .interference-grid {
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
    font-size: 1.25rem;
    font-weight: 600;
  }

  .quality-badge, .connected-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .connected-badge {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .quality-badge.excellent, .excellent {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .quality-badge.good, .good {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--success);
  }

  .quality-badge.fair, .fair {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .quality-badge.weak, .quality-badge.very-weak, .weak, .very-weak {
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

  .gauge-fill.excellent { background-color: var(--success); }
  .gauge-fill.good { background-color: #22c55e; }
  .gauge-fill.fair { background-color: var(--warning); }
  .gauge-fill.weak, .gauge-fill.very-weak { background-color: var(--error); }

  .gauge-value {
    text-align: center;
    font-size: 1.25rem;
    font-weight: 600;
    font-family: ui-monospace, monospace;
  }

  .channel-num {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent);
  }

  /* Network list styles */
  .network-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .network-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    gap: 1rem;
  }

  .network-item.connected {
    border: 1px solid var(--success);
    background-color: rgba(16, 185, 129, 0.05);
  }

  .network-info {
    flex: 1;
    min-width: 0;
  }

  .network-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .network-name .ssid {
    font-size: 1rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .network-details {
    display: flex;
    gap: 0.75rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .network-signal {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.25rem;
  }

  .signal-value {
    font-family: ui-monospace, monospace;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .mini-gauge {
    width: 60px;
    height: 4px;
    background-color: var(--bg-secondary);
    border-radius: 2px;
    overflow: hidden;
  }

  .mini-gauge-fill {
    height: 100%;
    border-radius: 2px;
  }

  .mini-gauge-fill.excellent { background-color: var(--success); }
  .mini-gauge-fill.good { background-color: #22c55e; }
  .mini-gauge-fill.fair { background-color: var(--warning); }
  .mini-gauge-fill.weak, .mini-gauge-fill.very-weak { background-color: var(--error); }

  /* Channel list styles */
  .channel-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .channel-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    flex-wrap: wrap;
  }

  .channel-item.current {
    border: 1px solid var(--accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .channel-item.recommended {
    border: 1px solid var(--success);
  }

  .channel-number {
    font-weight: 600;
    min-width: 2rem;
  }

  .channel-networks {
    color: var(--text-secondary);
    font-size: 0.875rem;
    flex: 1;
  }

  .channel-interference {
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .current-badge, .recommended-badge, .dfs-badge {
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .current-badge {
    background-color: rgba(59, 130, 246, 0.2);
    color: var(--accent);
  }

  .recommended-badge {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .dfs-badge {
    background-color: rgba(245, 158, 11, 0.2);
    color: var(--warning);
  }

  /* Interference styles */
  .good-message {
    color: var(--success);
    margin: 0;
  }

  .overlap-list {
    margin: 0;
    padding-left: 1.25rem;
    color: var(--text-secondary);
  }

  .overlap-list li {
    margin-bottom: 0.25rem;
  }

  .recommendations-card {
    grid-column: 1 / -1;
  }

  .recommendations-list {
    margin: 0;
    padding-left: 1.25rem;
  }

  .recommendations-list li {
    margin-bottom: 0.5rem;
  }

  @media (max-width: 600px) {
    .tabs {
      flex-wrap: wrap;
    }

    .tab {
      flex: 1;
      min-width: 80px;
      text-align: center;
    }
  }
</style>
