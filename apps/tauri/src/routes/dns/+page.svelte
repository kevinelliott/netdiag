<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface DnsResult {
    hostname: string;
    addresses: string[];
    duration_ms: number;
    error?: string;
  }

  let hostname = $state('google.com');
  let result = $state<DnsResult | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let history = $state<DnsResult[]>([]);

  async function runLookup() {
    if (!hostname.trim()) return;

    loading = true;
    error = null;
    result = null;

    try {
      result = await invoke<DnsResult>('dns_lookup', {
        hostname: hostname.trim(),
      });
      if (result.error) {
        error = result.error;
      } else {
        // Add to history (keep last 10)
        history = [result, ...history.slice(0, 9)];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      runLookup();
    }
  }

  function lookupFromHistory(h: DnsResult) {
    hostname = h.hostname;
    result = h;
  }

  function clearHistory() {
    history = [];
  }
</script>

<div class="page">
  <h1 class="title">DNS Lookup</h1>

  <div class="card form-card">
    <div class="form-row">
      <div class="form-group hostname-group">
        <label for="hostname">Hostname</label>
        <input
          id="hostname"
          type="text"
          class="input"
          bind:value={hostname}
          onkeydown={handleKeydown}
          placeholder="google.com"
        />
      </div>
      <button
        class="btn btn-primary"
        onclick={runLookup}
        disabled={loading || !hostname.trim()}
      >
        {loading ? 'Looking up...' : 'Lookup'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if result && !result.error}
    <div class="card result-card">
      <h2 class="card-title">Results for {result.hostname}</h2>

      <div class="result-meta">
        <span class="duration">Resolved in {result.duration_ms.toFixed(2)} ms</span>
        <span class="count">{result.addresses.length} address{result.addresses.length !== 1 ? 'es' : ''}</span>
      </div>

      <div class="addresses">
        <h3 class="section-title">Resolved Addresses</h3>
        <ul class="address-list">
          {#each result.addresses as addr}
            <li class="address-item">
              <span class="addr mono">{addr}</span>
              <span class="type-badge">{addr.includes(':') ? 'IPv6' : 'IPv4'}</span>
            </li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}

  {#if history.length > 0}
    <div class="history-section">
      <div class="history-header">
        <h2 class="section-title">Recent Lookups</h2>
        <button class="btn-link" onclick={clearHistory}>Clear</button>
      </div>
      <div class="history-list">
        {#each history as h}
          <button class="history-item" onclick={() => lookupFromHistory(h)}>
            <span class="history-hostname">{h.hostname}</span>
            <span class="history-addresses">{h.addresses.length} addr</span>
            <span class="history-time">{h.duration_ms.toFixed(1)} ms</span>
          </button>
        {/each}
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

  .title {
    margin: 0;
    font-size: 1.5rem;
  }

  .form-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-row {
    display: flex;
    gap: 1rem;
    align-items: end;
  }

  @media (max-width: 500px) {
    .form-row {
      flex-direction: column;
      align-items: stretch;
    }
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .hostname-group {
    flex: 1;
  }

  .form-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    color: var(--error);
  }

  .card-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
  }

  .result-meta {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .section-title {
    margin: 0 0 0.75rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .address-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .address-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
  }

  .addr {
    flex: 1;
    font-family: ui-monospace, monospace;
    font-size: 0.9375rem;
    word-break: break-all;
  }

  .type-badge {
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
  }

  .history-section {
    margin-top: 1rem;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .history-header .section-title {
    margin: 0;
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0;
  }

  .btn-link:hover {
    text-decoration: underline;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    width: 100%;
  }

  .history-item:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .history-hostname {
    flex: 1;
    font-weight: 500;
  }

  .history-addresses, .history-time {
    color: var(--text-secondary);
    font-size: 0.875rem;
    font-family: ui-monospace, monospace;
  }

  .mono {
    font-family: ui-monospace, monospace;
  }
</style>
