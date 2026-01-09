<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface CaptureDevice {
    name: string;
    description: string | null;
    addresses: string[];
    is_loopback: boolean;
    is_up: boolean;
  }

  interface CapturedPacket {
    timestamp: string;
    protocol: string;
    src_ip: string | null;
    dst_ip: string | null;
    src_port: number | null;
    dst_port: number | null;
    length: number;
    summary: string;
  }

  let isAvailable = $state(false);
  let devices = $state<CaptureDevice[]>([]);
  let packets = $state<CapturedPacket[]>([]);
  let selectedDevice = $state('');
  let filter = $state('');
  let maxPackets = $state(100);
  let isCapturing = $state(false);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  // Predefined filters
  const commonFilters = [
    { label: 'All Traffic', value: '' },
    { label: 'TCP Only', value: 'tcp' },
    { label: 'UDP Only', value: 'udp' },
    { label: 'ICMP Only', value: 'icmp' },
    { label: 'HTTP/HTTPS', value: 'tcp port 80 or tcp port 443' },
    { label: 'DNS', value: 'udp port 53' },
    { label: 'SSH', value: 'tcp port 22' },
  ];

  onMount(async () => {
    await checkAvailability();
    if (isAvailable) {
      await loadDevices();
    }
    isLoading = false;
  });

  async function checkAvailability() {
    try {
      isAvailable = await invoke<boolean>('is_capture_available');
    } catch (e) {
      isAvailable = false;
    }
  }

  async function loadDevices() {
    try {
      devices = await invoke<CaptureDevice[]>('list_capture_devices');
      // Auto-select first non-loopback, up device
      const defaultDevice = devices.find(d => d.is_up && !d.is_loopback);
      if (defaultDevice) {
        selectedDevice = defaultDevice.name;
      } else if (devices.length > 0) {
        selectedDevice = devices[0].name;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function startCapture() {
    if (!selectedDevice) {
      error = 'Please select a capture device';
      return;
    }

    isCapturing = true;
    error = null;
    packets = [];

    try {
      const result = await invoke<CapturedPacket[]>('capture_packets', {
        device: selectedDevice,
        filter: filter || null,
        maxPackets: maxPackets,
      });
      packets = result;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isCapturing = false;
    }
  }

  function clearPackets() {
    packets = [];
  }

  function getProtocolColor(protocol: string): string {
    switch (protocol.toLowerCase()) {
      case 'tcp':
        return 'var(--accent)';
      case 'udp':
        return 'var(--warning)';
      case 'icmp':
      case 'icmpv6':
        return 'var(--success)';
      case 'arp':
        return 'var(--info)';
      default:
        return 'var(--text-secondary)';
    }
  }

  function formatTimestamp(ts: string): string {
    try {
      const date = new Date(ts);
      return date.toLocaleTimeString('en-US', {
        hour12: false,
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        fractionalSecondDigits: 3,
      });
    } catch {
      return ts;
    }
  }
</script>

<div class="page">
  <h1>Packet Capture</h1>

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <span>Checking capture availability...</span>
    </div>
  {:else if !isAvailable}
    <div class="unavailable">
      <div class="unavailable-icon">🔒</div>
      <h2>Packet Capture Not Available</h2>
      <p>Packet capture requires elevated privileges and is only available on desktop platforms (macOS, Linux, Windows).</p>
      <div class="requirements">
        <h3>Requirements:</h3>
        <ul>
          <li><strong>macOS:</strong> Run with sudo or grant app special network permissions</li>
          <li><strong>Linux:</strong> Run with sudo or use setcap on the binary</li>
          <li><strong>Windows:</strong> Install WinPcap or Npcap and run as Administrator</li>
        </ul>
      </div>
    </div>
  {:else}
    <div class="capture-controls">
      <div class="control-row">
        <div class="control-group">
          <label for="device">Capture Device</label>
          <select id="device" bind:value={selectedDevice} disabled={isCapturing}>
            {#each devices as device}
              <option value={device.name}>
                {device.name}
                {#if device.description}({device.description}){/if}
                {#if !device.is_up}[DOWN]{/if}
                {#if device.is_loopback}[LOOPBACK]{/if}
              </option>
            {/each}
          </select>
        </div>

        <div class="control-group">
          <label for="filter">BPF Filter</label>
          <div class="filter-input">
            <input
              type="text"
              id="filter"
              bind:value={filter}
              placeholder="e.g., tcp port 80"
              disabled={isCapturing}
            />
            <select
              class="filter-preset"
              onchange={(e) => { filter = (e.target as HTMLSelectElement).value; }}
              disabled={isCapturing}
            >
              <option value="">Presets...</option>
              {#each commonFilters as preset}
                <option value={preset.value}>{preset.label}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="control-group">
          <label for="max-packets">Max Packets</label>
          <input
            type="number"
            id="max-packets"
            bind:value={maxPackets}
            min="1"
            max="10000"
            disabled={isCapturing}
          />
        </div>
      </div>

      <div class="button-row">
        <button
          class="btn btn-primary"
          onclick={startCapture}
          disabled={isCapturing || !selectedDevice}
        >
          {#if isCapturing}
            <span class="spinner small"></span>
            Capturing...
          {:else}
            Start Capture
          {/if}
        </button>
        <button
          class="btn btn-secondary"
          onclick={clearPackets}
          disabled={isCapturing || packets.length === 0}
        >
          Clear
        </button>
      </div>
    </div>

    {#if error}
      <div class="error">
        <strong>Error:</strong> {error}
      </div>
    {/if}

    <div class="packet-list">
      <div class="packet-header">
        <span class="col-time">Time</span>
        <span class="col-protocol">Protocol</span>
        <span class="col-source">Source</span>
        <span class="col-dest">Destination</span>
        <span class="col-length">Length</span>
        <span class="col-info">Info</span>
      </div>

      {#if packets.length === 0}
        <div class="empty-state">
          {#if isCapturing}
            <div class="spinner"></div>
            <span>Waiting for packets...</span>
          {:else}
            <span>No packets captured. Click "Start Capture" to begin.</span>
          {/if}
        </div>
      {:else}
        <div class="packet-rows">
          {#each packets as packet, i}
            <div class="packet-row" class:even={i % 2 === 0}>
              <span class="col-time">{formatTimestamp(packet.timestamp)}</span>
              <span class="col-protocol">
                <span class="protocol-badge" style="background-color: {getProtocolColor(packet.protocol)}">
                  {packet.protocol}
                </span>
              </span>
              <span class="col-source">
                {packet.src_ip || '-'}
                {#if packet.src_port}:{packet.src_port}{/if}
              </span>
              <span class="col-dest">
                {packet.dst_ip || '-'}
                {#if packet.dst_port}:{packet.dst_port}{/if}
              </span>
              <span class="col-length">{packet.length}</span>
              <span class="col-info" title={packet.summary}>{packet.summary}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="capture-stats">
      <span>Captured: <strong>{packets.length}</strong> packets</span>
      {#if packets.length > 0}
        <span>Total bytes: <strong>{packets.reduce((sum, p) => sum + p.length, 0).toLocaleString()}</strong></span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
  }

  h1 {
    margin-bottom: 1.5rem;
    color: var(--text-primary);
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--text-secondary);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .spinner.small {
    width: 16px;
    height: 16px;
    border-width: 2px;
    display: inline-block;
    margin-right: 0.5rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .unavailable {
    text-align: center;
    padding: 3rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    border: 1px solid var(--border);
  }

  .unavailable-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .unavailable h2 {
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .unavailable p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }

  .requirements {
    text-align: left;
    background: var(--bg-tertiary);
    padding: 1rem 1.5rem;
    border-radius: 0.375rem;
    max-width: 500px;
    margin: 0 auto;
  }

  .requirements h3 {
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .requirements ul {
    margin: 0;
    padding-left: 1.25rem;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .requirements li {
    margin-bottom: 0.25rem;
  }

  .capture-controls {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .control-row {
    display: grid;
    grid-template-columns: 1fr 2fr 120px;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  @media (max-width: 768px) {
    .control-row {
      grid-template-columns: 1fr;
    }
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .control-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .control-group select,
  .control-group input {
    padding: 0.5rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .filter-input {
    display: flex;
    gap: 0.5rem;
  }

  .filter-input input {
    flex: 1;
  }

  .filter-preset {
    width: 120px;
  }

  .button-row {
    display: flex;
    gap: 0.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-primary);
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    color: var(--error);
    padding: 0.75rem 1rem;
    border-radius: 0.375rem;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .packet-list {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .packet-header {
    display: grid;
    grid-template-columns: 100px 80px 180px 180px 70px 1fr;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: var(--bg-tertiary);
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
  }

  @media (max-width: 900px) {
    .packet-header,
    .packet-row {
      grid-template-columns: 80px 60px 1fr 1fr 50px;
    }
    .col-info {
      display: none;
    }
  }

  .packet-rows {
    max-height: 500px;
    overflow-y: auto;
  }

  .packet-row {
    display: grid;
    grid-template-columns: 100px 80px 180px 180px 70px 1fr;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    border-bottom: 1px solid var(--border);
  }

  .packet-row.even {
    background: var(--bg-tertiary);
  }

  .packet-row:last-child {
    border-bottom: none;
  }

  .col-time {
    color: var(--text-secondary);
  }

  .col-source,
  .col-dest {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-length {
    text-align: right;
    color: var(--text-secondary);
  }

  .col-info {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .protocol-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: white;
    text-transform: uppercase;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--text-secondary);
  }

  .capture-stats {
    display: flex;
    gap: 1.5rem;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    margin-top: 1rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .capture-stats strong {
    color: var(--text-primary);
  }
</style>
