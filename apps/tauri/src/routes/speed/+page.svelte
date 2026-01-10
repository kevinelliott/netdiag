<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { tick } from 'svelte';

  interface SpeedTestResult {
    download_mbps?: number | null;
    upload_mbps?: number | null;
    latency_ms?: number | null;
    jitter_ms?: number | null;
    server_name: string;
    server_location?: string | null;
    test_duration_secs: number;
    buffer_bloat_grade?: string | null;
    consistency_rating?: string | null;
  }

  let duration = $state(10);
  let connections = $state(4);
  let testDownload = $state(true);
  let testUpload = $state(true);
  let result = $state<SpeedTestResult | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let testPhase = $state<string>('');
  let resultCard: HTMLElement | null = null;

  async function runSpeedTest() {
    loading = true;
    error = null;
    result = null;
    testPhase = 'Initializing...';

    try {
      testPhase = 'Running speed test...';
      const response = await invoke<SpeedTestResult>('run_speed_test', {
        durationSecs: duration,
        connections,
        testDownload,
        testUpload,
      });
      console.log('Speed test result:', response);

      // Ensure we have a valid result
      if (response) {
        result = response;
        // Wait for DOM update then scroll to results
        await tick();
        if (resultCard) {
          resultCard.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
      } else {
        error = 'Speed test completed but returned no data';
      }
      testPhase = '';
    } catch (e) {
      console.error('Speed test error:', e);
      error = e instanceof Error ? e.message : String(e);
      testPhase = '';
    } finally {
      loading = false;
    }
  }

  function formatSpeed(mbps: number | undefined): string {
    if (mbps === undefined || mbps === null) return '-';
    if (mbps >= 1000) {
      return `${(mbps / 1000).toFixed(2)} Gbps`;
    }
    return `${mbps.toFixed(2)} Mbps`;
  }

  function getSpeedQuality(mbps: number | undefined): { label: string; class: string } {
    if (mbps === undefined || mbps === null) return { label: '-', class: '' };
    if (mbps >= 500) return { label: 'Excellent', class: 'excellent' };
    if (mbps >= 100) return { label: 'Very Good', class: 'very-good' };
    if (mbps >= 50) return { label: 'Good', class: 'good' };
    if (mbps >= 25) return { label: 'Fair', class: 'fair' };
    if (mbps >= 10) return { label: 'Poor', class: 'poor' };
    return { label: 'Very Poor', class: 'very-poor' };
  }

  function getGradeClass(grade: string | undefined): string {
    if (!grade) return '';
    switch (grade) {
      case 'A': return 'excellent';
      case 'B': return 'very-good';
      case 'C': return 'good';
      case 'D': return 'fair';
      case 'F': return 'poor';
      default: return '';
    }
  }

  function getConsistencyClass(rating: string | undefined): string {
    if (!rating) return '';
    switch (rating) {
      case 'Excellent': return 'excellent';
      case 'Good': return 'very-good';
      case 'Fair': return 'fair';
      case 'Poor': return 'poor';
      case 'Very Poor': return 'very-poor';
      default: return '';
    }
  }
</script>

<div class="page">
  <h1 class="title">Speed Test</h1>

  <div class="card form-card">
    <div class="form-row">
      <div class="form-group">
        <label for="duration">Duration (seconds)</label>
        <input
          id="duration"
          type="number"
          class="input"
          bind:value={duration}
          min="5"
          max="60"
          step="5"
        />
      </div>
      <div class="form-group">
        <label for="connections">Connections</label>
        <input
          id="connections"
          type="number"
          class="input"
          bind:value={connections}
          min="1"
          max="16"
        />
      </div>
    </div>

    <div class="checkbox-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={testDownload} />
        Test Download
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={testUpload} />
        Test Upload
      </label>
    </div>

    <button
      class="btn btn-primary"
      onclick={runSpeedTest}
      disabled={loading || (!testDownload && !testUpload)}
    >
      {loading ? testPhase || 'Testing...' : 'Start Speed Test'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if loading}
    <div class="card loading-card">
      <div class="spinner"></div>
      <p class="loading-text">{testPhase}</p>
      <p class="loading-hint">This may take up to {duration * 2 + 10} seconds...</p>
    </div>
  {/if}

  {#if result && !loading}
    <div class="card result-card" bind:this={resultCard}>
      <h2 class="card-title">Speed Test Results</h2>
      <p class="server-info">
        Server: {result.server_name || 'Unknown'}
        {#if result.server_location}
          ({result.server_location})
        {/if}
      </p>

      <div class="speed-display">
        {#if result.download_mbps != null}
          <div class="speed-block download">
            <span class="speed-label">Download</span>
            <span class="speed-value">{formatSpeed(result.download_mbps)}</span>
            <span class="speed-quality {getSpeedQuality(result.download_mbps).class}">
              {getSpeedQuality(result.download_mbps).label}
            </span>
          </div>
        {/if}

        {#if result.upload_mbps != null}
          <div class="speed-block upload">
            <span class="speed-label">Upload</span>
            <span class="speed-value">{formatSpeed(result.upload_mbps)}</span>
            <span class="speed-quality {getSpeedQuality(result.upload_mbps).class}">
              {getSpeedQuality(result.upload_mbps).label}
            </span>
          </div>
        {/if}
      </div>

      <div class="stats-grid">
        {#if result.latency_ms != null}
          <div class="stat">
            <span class="stat-label">Latency</span>
            <span class="stat-value">{result.latency_ms.toFixed(2)} ms</span>
          </div>
        {/if}

        {#if result.jitter_ms != null}
          <div class="stat">
            <span class="stat-label">Jitter</span>
            <span class="stat-value">{result.jitter_ms.toFixed(2)} ms</span>
          </div>
        {/if}

        {#if result.test_duration_secs != null}
          <div class="stat">
            <span class="stat-label">Test Duration</span>
            <span class="stat-value">{result.test_duration_secs.toFixed(1)}s</span>
          </div>
        {/if}
      </div>

      {#if result.buffer_bloat_grade || result.consistency_rating}
        <div class="analysis-section">
          <h3 class="section-title">Connection Analysis</h3>
          <div class="analysis-grid">
            {#if result.buffer_bloat_grade}
              <div class="analysis-item">
                <span class="analysis-label">Buffer Bloat</span>
                <span class="analysis-badge {getGradeClass(result.buffer_bloat_grade)}">
                  Grade {result.buffer_bloat_grade}
                </span>
              </div>
            {/if}

            {#if result.consistency_rating}
              <div class="analysis-item">
                <span class="analysis-label">Consistency</span>
                <span class="analysis-badge {getConsistencyClass(result.consistency_rating)}">
                  {result.consistency_rating}
                </span>
              </div>
            {/if}
          </div>
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
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  @media (max-width: 500px) {
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

  .checkbox-row {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-weight: 500;
  }

  .checkbox-label input {
    width: 18px;
    height: 18px;
    cursor: pointer;
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
    padding: 3rem;
    gap: 1rem;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-text {
    margin: 0;
    font-weight: 600;
    font-size: 1.125rem;
    color: var(--text-primary);
  }

  .loading-hint {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .card-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
  }

  .server-info {
    margin: 0 0 1.5rem 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .speed-display {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .speed-block {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.5rem;
    border-radius: 0.5rem;
    background-color: var(--bg-tertiary);
  }

  .speed-block.download {
    border-left: 4px solid var(--accent);
  }

  .speed-block.upload {
    border-left: 4px solid var(--success);
  }

  .speed-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .speed-value {
    font-size: 2rem;
    font-weight: 700;
    font-family: ui-monospace, monospace;
    color: var(--text-primary);
  }

  .speed-quality {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 1rem;
    padding: 1rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    margin-bottom: 1.5rem;
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
    font-size: 1.25rem;
    font-weight: 600;
    font-family: ui-monospace, monospace;
  }

  .analysis-section {
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .section-title {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .analysis-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }

  .analysis-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .analysis-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .analysis-badge {
    display: inline-block;
    padding: 0.375rem 0.75rem;
    border-radius: 0.375rem;
    font-weight: 600;
    font-size: 0.875rem;
    width: fit-content;
  }

  .excellent {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .very-good {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--success);
  }

  .good {
    background-color: rgba(59, 130, 246, 0.15);
    color: var(--accent);
  }

  .fair {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .poor {
    background-color: rgba(239, 68, 68, 0.15);
    color: var(--error);
  }

  .very-poor {
    background-color: rgba(239, 68, 68, 0.2);
    color: var(--error);
  }
</style>
