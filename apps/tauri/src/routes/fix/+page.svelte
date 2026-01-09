<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface FixActionInfo {
    id: string;
    name: string;
    description: string;
    severity: string;
    category: string;
    reversible: boolean;
    estimated_time_secs: number;
    prerequisites: string[];
  }

  interface FixResultInfo {
    action_id: string;
    success: boolean;
    message: string | null;
    error: string | null;
    duration_ms: number;
    rollback_id: string | null;
  }

  interface InterfaceInfo {
    name: string;
    friendly_name: string | null;
    is_up: boolean;
    is_default: boolean;
  }

  let available = $state(true);
  let fixes = $state<FixActionInfo[]>([]);
  let interfaces = $state<InterfaceInfo[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedFix = $state<string | null>(null);
  let selectedInterface = $state<string | null>(null);
  let dryRun = $state(true);
  let result = $state<FixResultInfo | null>(null);
  let applying = $state(false);

  onMount(async () => {
    await checkAvailability();
    if (available) {
      await loadFixes();
      await loadInterfaces();
    }
  });

  async function checkAvailability() {
    try {
      available = await invoke<boolean>('is_autofix_available');
    } catch {
      available = false;
    }
  }

  async function loadFixes() {
    loading = true;
    error = null;
    try {
      fixes = await invoke<FixActionInfo[]>('get_available_fixes');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function loadInterfaces() {
    try {
      interfaces = await invoke<InterfaceInfo[]>('get_interfaces');
      // Pre-select default interface
      const defaultIface = interfaces.find(i => i.is_default && i.is_up);
      if (defaultIface) {
        selectedInterface = defaultIface.name;
      } else if (interfaces.length > 0) {
        selectedInterface = interfaces[0].name;
      }
    } catch {
      // Ignore errors loading interfaces
    }
  }

  async function applyFix() {
    if (!selectedFix) return;

    applying = true;
    result = null;
    error = null;

    try {
      result = await invoke<FixResultInfo>('apply_fix', {
        fixType: selectedFix,
        interface: needsInterface(selectedFix) ? selectedInterface : null,
        dryRun,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      applying = false;
    }
  }

  function needsInterface(fixType: string | null): boolean {
    return ['reset_adapter', 'renew_dhcp', 'reconnect_wifi'].includes(fixType || '');
  }

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'low': return 'var(--success)';
      case 'medium': return 'var(--warning)';
      case 'high': return 'var(--error)';
      case 'critical': return '#ff0000';
      default: return 'var(--text-secondary)';
    }
  }

  function getCategoryIcon(category: string): string {
    switch (category) {
      case 'dns': return 'DNS';
      case 'adapter': return 'NIC';
      case 'tcp_ip': return 'TCP';
      case 'wifi': return 'WiFi';
      case 'service': return 'SVC';
      default: return 'FIX';
    }
  }
</script>

<div class="page">
  <h1 class="title">Auto-Fix Network Issues</h1>

  {#if !available}
    <div class="card unavailable-card">
      <div class="unavailable-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
        </svg>
      </div>
      <h2>Not Available</h2>
      <p>Auto-fix features are not available on mobile platforms. These features require system-level access that is only available on desktop operating systems.</p>
    </div>
  {:else}
    {#if error}
      <div class="error-banner">Error: {error}</div>
    {/if}

    {#if loading}
      <div class="card loading-card">
        <div class="spinner"></div>
        <p>Loading available fixes...</p>
      </div>
    {:else}
      <div class="card warning-card">
        <div class="warning-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
        </div>
        <div class="warning-content">
          <h3>Caution</h3>
          <p>Some fixes may temporarily disrupt your network connection. Use <strong>Dry Run</strong> mode first to see what changes would be made without actually applying them.</p>
        </div>
      </div>

      <div class="card fixes-card">
        <h2 class="card-title">Available Fixes</h2>
        <div class="fixes-list">
          {#each fixes as fix}
            <button
              class="fix-item"
              class:selected={selectedFix === fix.id.split('-')[0] || getFixType(fix) === selectedFix}
              onclick={() => selectedFix = getFixType(fix)}
            >
              <div class="fix-category" style="background-color: {getSeverityColor(fix.severity)}20; color: {getSeverityColor(fix.severity)}">
                {getCategoryIcon(fix.category)}
              </div>
              <div class="fix-content">
                <div class="fix-header">
                  <span class="fix-name">{fix.name}</span>
                  <span class="fix-severity" style="color: {getSeverityColor(fix.severity)}">
                    {fix.severity}
                  </span>
                </div>
                <p class="fix-description">{fix.description}</p>
                <div class="fix-meta">
                  <span class="fix-time">~{fix.estimated_time_secs}s</span>
                  {#if fix.reversible}
                    <span class="fix-reversible">Reversible</span>
                  {/if}
                  {#if fix.prerequisites.length > 0}
                    <span class="fix-prereq">Requires: {fix.prerequisites.join(', ')}</span>
                  {/if}
                </div>
              </div>
            </button>
          {/each}

          <!-- Interface-specific fixes -->
          <button
            class="fix-item"
            class:selected={selectedFix === 'reset_adapter'}
            onclick={() => selectedFix = 'reset_adapter'}
          >
            <div class="fix-category" style="background-color: rgba(245, 158, 11, 0.2); color: var(--warning)">
              NIC
            </div>
            <div class="fix-content">
              <div class="fix-header">
                <span class="fix-name">Reset Network Adapter</span>
                <span class="fix-severity" style="color: var(--warning)">medium</span>
              </div>
              <p class="fix-description">Disables and re-enables a network adapter</p>
              <div class="fix-meta">
                <span class="fix-time">~10s</span>
                <span class="fix-prereq">Requires: Admin</span>
              </div>
            </div>
          </button>

          <button
            class="fix-item"
            class:selected={selectedFix === 'renew_dhcp'}
            onclick={() => selectedFix = 'renew_dhcp'}
          >
            <div class="fix-category" style="background-color: rgba(245, 158, 11, 0.2); color: var(--warning)">
              DHCP
            </div>
            <div class="fix-content">
              <div class="fix-header">
                <span class="fix-name">Renew DHCP Lease</span>
                <span class="fix-severity" style="color: var(--warning)">medium</span>
              </div>
              <p class="fix-description">Releases and renews the DHCP lease to get a fresh IP</p>
              <div class="fix-meta">
                <span class="fix-time">~5s</span>
              </div>
            </div>
          </button>
        </div>
      </div>

      {#if selectedFix}
        <div class="card apply-card">
          <h2 class="card-title">Apply Fix</h2>

          {#if needsInterface(selectedFix)}
            <div class="form-group">
              <label class="form-label">Network Interface</label>
              <select class="form-select" bind:value={selectedInterface}>
                {#each interfaces.filter(i => i.is_up) as iface}
                  <option value={iface.name}>
                    {iface.friendly_name || iface.name}
                    {iface.is_default ? ' (Default)' : ''}
                  </option>
                {/each}
              </select>
            </div>
          {/if}

          <div class="form-group">
            <label class="checkbox-option">
              <input type="checkbox" bind:checked={dryRun} />
              <span>Dry Run (preview changes without applying)</span>
            </label>
          </div>

          <button class="btn btn-primary" onclick={applyFix} disabled={applying}>
            {#if applying}
              <span class="btn-spinner"></span>
              Applying...
            {:else if dryRun}
              Preview Changes
            {:else}
              Apply Fix
            {/if}
          </button>
        </div>
      {/if}

      {#if result}
        <div class="card result-card" class:success={result.success} class:failure={!result.success}>
          <div class="result-header">
            {#if result.success}
              <div class="result-icon success">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              </div>
              <span class="result-title">
                {dryRun ? 'Dry Run Complete' : 'Fix Applied Successfully'}
              </span>
            {:else}
              <div class="result-icon failure">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </div>
              <span class="result-title">Fix Failed</span>
            {/if}
          </div>
          <div class="result-body">
            {#if result.message}
              <p class="result-message">{result.message}</p>
            {/if}
            {#if result.error}
              <p class="result-error">{result.error}</p>
            {/if}
            <p class="result-duration">Duration: {result.duration_ms}ms</p>
          </div>
        </div>
      {/if}
    {/if}
  {/if}
</div>

<script context="module" lang="ts">
  function getFixType(fix: { name: string }): string {
    const name = fix.name.toLowerCase();
    if (name.includes('dns') && name.includes('cache')) return 'flush_dns_cache';
    if (name.includes('tcp/ip')) return 'reset_tcp_ip';
    if (name.includes('network service')) return 'restart_network_service';
    return fix.name.toLowerCase().replace(/\s+/g, '_');
  }
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .title {
    margin: 0;
    font-size: 1.5rem;
  }

  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .unavailable-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
    gap: 1rem;
  }

  .unavailable-icon {
    width: 64px;
    height: 64px;
    color: var(--text-secondary);
  }

  .unavailable-icon svg {
    width: 100%;
    height: 100%;
  }

  .unavailable-card h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--text-primary);
  }

  .unavailable-card p {
    margin: 0;
    color: var(--text-secondary);
    max-width: 400px;
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    color: var(--error);
  }

  .loading-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    gap: 1rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .warning-card {
    display: flex;
    gap: 1rem;
    background-color: rgba(245, 158, 11, 0.1);
    border-left: 4px solid var(--warning);
  }

  .warning-icon {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    color: var(--warning);
  }

  .warning-content h3 {
    margin: 0 0 0.25rem 0;
    font-size: 0.875rem;
    color: var(--warning);
  }

  .warning-content p {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .fixes-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .fix-item {
    display: flex;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: 0.5rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    width: 100%;
  }

  .fix-item:hover {
    background-color: var(--bg-secondary);
  }

  .fix-item.selected {
    border-color: var(--accent);
    background-color: rgba(59, 130, 246, 0.1);
  }

  .fix-category {
    flex-shrink: 0;
    width: 48px;
    height: 48px;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.75rem;
  }

  .fix-content {
    flex: 1;
    min-width: 0;
  }

  .fix-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .fix-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .fix-severity {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .fix-description {
    margin: 0 0 0.5rem 0;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .fix-meta {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .fix-time,
  .fix-reversible,
  .fix-prereq {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .fix-reversible {
    color: var(--success);
  }

  .apply-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .form-label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .form-select {
    padding: 0.5rem;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .checkbox-option input {
    width: 18px;
    height: 18px;
  }

  .btn-spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-right: 0.5rem;
  }

  .result-card {
    border-left: 4px solid var(--border);
  }

  .result-card.success {
    border-left-color: var(--success);
    background-color: rgba(16, 185, 129, 0.05);
  }

  .result-card.failure {
    border-left-color: var(--error);
    background-color: rgba(239, 68, 68, 0.05);
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .result-icon {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
  }

  .result-icon.success {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .result-icon.failure {
    background-color: rgba(239, 68, 68, 0.2);
    color: var(--error);
  }

  .result-icon svg {
    width: 16px;
    height: 16px;
  }

  .result-title {
    font-weight: 600;
    font-size: 1rem;
  }

  .result-body {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .result-message {
    margin: 0;
    color: var(--text-primary);
  }

  .result-error {
    margin: 0;
    color: var(--error);
  }

  .result-duration {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* Mobile adjustments */
  @media (max-width: 768px) {
    .fix-item {
      flex-direction: column;
    }

    .fix-category {
      width: 100%;
      height: 32px;
    }
  }
</style>
