<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface TracerouteHop {
    hop: number;
    address?: string;
    hostname?: string;
    rtt_ms: (number | null)[];
    is_timeout: boolean;
  }

  interface TracerouteResult {
    target: string;
    resolved_ip?: string;
    hops: TracerouteHop[];
    reached_destination: boolean;
    error?: string;
  }

  let target = $state('google.com');
  let maxHops = $state(30);
  let timeout = $state(2000);
  let result = $state<TracerouteResult | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function runTraceroute() {
    if (!target.trim()) return;

    loading = true;
    error = null;
    result = null;

    try {
      result = await invoke<TracerouteResult>('traceroute_target', {
        target: target.trim(),
        maxHops,
        timeoutMs: timeout,
      });
      if (result.error) {
        error = result.error;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      runTraceroute();
    }
  }

  function formatRtt(rtt: number | null): string {
    if (rtt === null) return '*';
    return `${rtt.toFixed(1)}`;
  }
</script>

<div class="page">
  <h1 class="title">Traceroute</h1>

  <div class="card form-card">
    <div class="form-row">
      <div class="form-group target-group">
        <label for="target">Target (hostname or IP)</label>
        <input
          id="target"
          type="text"
          class="input"
          bind:value={target}
          onkeydown={handleKeydown}
          placeholder="google.com or 8.8.8.8"
        />
      </div>
      <div class="form-group">
        <label for="maxHops">Max Hops</label>
        <input
          id="maxHops"
          type="number"
          class="input"
          bind:value={maxHops}
          min="1"
          max="64"
        />
      </div>
      <div class="form-group">
        <label for="timeout">Timeout (ms)</label>
        <input
          id="timeout"
          type="number"
          class="input"
          bind:value={timeout}
          min="100"
          max="10000"
          step="100"
        />
      </div>
    </div>
    <button
      class="btn btn-primary"
      onclick={runTraceroute}
      disabled={loading || !target.trim()}
    >
      {loading ? 'Tracing...' : 'Start Traceroute'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if result && !result.error}
    <div class="card result-card">
      <div class="result-header">
        <h2 class="card-title">Route to {result.target}</h2>
        {#if result.resolved_ip && result.resolved_ip !== result.target}
          <span class="resolved">({result.resolved_ip})</span>
        {/if}
      </div>

      <div class="status-row">
        {#if result.reached_destination}
          <span class="status-badge success">Destination Reached</span>
        {:else}
          <span class="status-badge warning">Destination Not Reached</span>
        {/if}
        <span class="hop-count">{result.hops.length} hops</span>
      </div>

      <table class="hop-table">
        <thead>
          <tr>
            <th class="hop-col">Hop</th>
            <th class="addr-col">Address</th>
            <th class="host-col">Hostname</th>
            <th class="rtt-col">RTT 1</th>
            <th class="rtt-col">RTT 2</th>
            <th class="rtt-col">RTT 3</th>
          </tr>
        </thead>
        <tbody>
          {#each result.hops as hop}
            <tr class:timeout={hop.is_timeout}>
              <td class="hop-num">{hop.hop}</td>
              <td class="addr mono">
                {#if hop.address}
                  {hop.address}
                {:else}
                  <span class="timeout-text">*</span>
                {/if}
              </td>
              <td class="host">
                {#if hop.hostname}
                  {hop.hostname}
                {:else if hop.address}
                  -
                {:else}
                  <span class="timeout-text">*</span>
                {/if}
              </td>
              {#each [0, 1, 2] as i}
                <td class="rtt mono">
                  {#if hop.rtt_ms[i] !== undefined}
                    <span class:timeout-val={hop.rtt_ms[i] === null}>
                      {formatRtt(hop.rtt_ms[i])}
                    </span>
                    {#if hop.rtt_ms[i] !== null}
                      <span class="unit">ms</span>
                    {/if}
                  {:else}
                    -
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
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
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 1rem;
    align-items: end;
  }

  @media (max-width: 600px) {
    .form-row {
      grid-template-columns: 1fr;
    }
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .form-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .form-group input[type="number"] {
    width: 100px;
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    color: var(--error);
  }

  .result-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .card-title {
    margin: 0;
    font-size: 1.125rem;
  }

  .resolved {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .status-badge.success {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--success);
  }

  .status-badge.warning {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .hop-count {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .hop-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .hop-table th {
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-bottom: 2px solid var(--border);
    color: var(--text-secondary);
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .hop-table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .hop-table tr:last-child td {
    border-bottom: none;
  }

  .hop-table tr.timeout {
    opacity: 0.6;
  }

  .hop-col {
    width: 50px;
  }

  .addr-col {
    min-width: 120px;
  }

  .host-col {
    min-width: 150px;
  }

  .rtt-col {
    width: 80px;
    text-align: right;
  }

  .hop-num {
    font-weight: 600;
    color: var(--text-secondary);
  }

  .addr, .rtt {
    font-family: ui-monospace, monospace;
  }

  .host {
    color: var(--text-secondary);
    word-break: break-all;
  }

  .rtt {
    text-align: right;
  }

  .timeout-text, .timeout-val {
    color: var(--text-secondary);
  }

  .unit {
    color: var(--text-secondary);
    font-size: 0.75rem;
    margin-left: 2px;
  }

  .mono {
    font-family: ui-monospace, monospace;
  }
</style>
