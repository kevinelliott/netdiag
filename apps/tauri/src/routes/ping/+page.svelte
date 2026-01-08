<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

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

  let target = $state('8.8.8.8');
  let count = $state(4);
  let timeout = $state(2000);
  let result = $state<PingResult | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function runPing() {
    if (!target.trim()) return;

    loading = true;
    error = null;
    result = null;

    try {
      result = await invoke<PingResult>('ping_target', {
        target: target.trim(),
        count,
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
      runPing();
    }
  }

  function getQuality(result: PingResult): { label: string; class: string } {
    if (result.loss_percent >= 50) return { label: 'Very Poor', class: 'very-poor' };
    if (result.loss_percent >= 10) return { label: 'Poor', class: 'poor' };
    if (result.loss_percent > 0) return { label: 'Fair', class: 'fair' };
    if (!result.avg_ms) return { label: 'Unknown', class: '' };
    if (result.avg_ms > 200) return { label: 'Fair', class: 'fair' };
    if (result.avg_ms > 100) return { label: 'Good', class: 'good' };
    if (result.avg_ms > 50) return { label: 'Very Good', class: 'very-good' };
    return { label: 'Excellent', class: 'excellent' };
  }
</script>

<div class="page">
  <h1 class="title">Ping Test</h1>

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
          placeholder="8.8.8.8 or google.com"
        />
      </div>
      <div class="form-group">
        <label for="count">Count</label>
        <input
          id="count"
          type="number"
          class="input"
          bind:value={count}
          min="1"
          max="100"
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
          max="30000"
          step="100"
        />
      </div>
    </div>
    <button
      class="btn btn-primary"
      onclick={runPing}
      disabled={loading || !target.trim()}
    >
      {loading ? 'Pinging...' : 'Start Ping'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if result && !result.error}
    <div class="card result-card">
      <h2 class="card-title">Results for {result.target}</h2>
      {#if result.resolved_ip && result.resolved_ip !== result.target}
        <p class="resolved">Resolved to: {result.resolved_ip}</p>
      {/if}

      <div class="stats-grid">
        <div class="stat">
          <span class="stat-label">Sent</span>
          <span class="stat-value">{result.sent}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Received</span>
          <span class="stat-value">{result.received}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Lost</span>
          <span class="stat-value" class:status-error={result.lost > 0}>{result.lost}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Loss</span>
          <span class="stat-value" class:status-error={result.loss_percent > 0}>
            {result.loss_percent.toFixed(1)}%
          </span>
        </div>
      </div>

      {#if result.avg_ms !== null}
        <div class="latency-section">
          <h3 class="section-title">Latency Statistics</h3>
          <div class="latency-grid">
            <div class="latency-stat">
              <span class="latency-label">Min</span>
              <span class="latency-value">{result.min_ms?.toFixed(2) ?? '-'} ms</span>
            </div>
            <div class="latency-stat">
              <span class="latency-label">Avg</span>
              <span class="latency-value highlight">{result.avg_ms?.toFixed(2) ?? '-'} ms</span>
            </div>
            <div class="latency-stat">
              <span class="latency-label">Max</span>
              <span class="latency-value">{result.max_ms?.toFixed(2) ?? '-'} ms</span>
            </div>
            {#if result.jitter_ms !== null}
              <div class="latency-stat">
                <span class="latency-label">Jitter</span>
                <span class="latency-value">{result.jitter_ms?.toFixed(2) ?? '-'} ms</span>
              </div>
            {/if}
          </div>
        </div>

        <div class="quality-section">
          <span class="quality-label">Connection Quality:</span>
          <span class="quality-badge {getQuality(result).class}">
            {getQuality(result).label}
          </span>
        </div>
      {/if}
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

  .card-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
  }

  .resolved {
    margin: 0 0 1rem 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
    padding: 1rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    margin-bottom: 1.5rem;
  }

  @media (max-width: 500px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
  }

  .section-title {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .latency-section {
    margin-bottom: 1.5rem;
  }

  .latency-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 1rem;
  }

  .latency-stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .latency-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .latency-value {
    font-size: 1.125rem;
    font-weight: 500;
    font-family: ui-monospace, monospace;
  }

  .latency-value.highlight {
    color: var(--accent);
    font-weight: 600;
  }

  .quality-section {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .quality-label {
    font-weight: 500;
  }

  .quality-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-weight: 600;
    font-size: 0.875rem;
  }

  .quality-badge.excellent {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .quality-badge.very-good {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--success);
  }

  .quality-badge.good {
    background-color: rgba(59, 130, 246, 0.15);
    color: var(--accent);
  }

  .quality-badge.fair {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .quality-badge.poor {
    background-color: rgba(239, 68, 68, 0.15);
    color: var(--error);
  }

  .quality-badge.very-poor {
    background-color: rgba(239, 68, 68, 0.2);
    color: var(--error);
  }
</style>
