<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface DiagnosticTest {
    name: string;
    category: string;
    passed: boolean;
    message: string;
    details?: Record<string, unknown>;
    duration_ms: number;
  }

  interface DiagnosticsSummary {
    total_tests: number;
    passed: number;
    failed: number;
    warnings: number;
    overall_status: string;
  }

  interface DiagnosticsResult {
    tests: DiagnosticTest[];
    summary: DiagnosticsSummary;
    issues: string[];
    recommendations: string[];
  }

  let result = $state<DiagnosticsResult | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let quickMode = $state(false);
  let includeSpeed = $state(true);
  let includeWifi = $state(true);
  let currentTest = $state<string>('');

  async function runDiagnostics() {
    loading = true;
    error = null;
    result = null;
    currentTest = 'Starting diagnostics...';

    try {
      result = await invoke<DiagnosticsResult>('run_diagnostics', {
        quick: quickMode,
        includeSpeed: includeSpeed && !quickMode,
        includeWifi,
      });
      currentTest = '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      currentTest = '';
    } finally {
      loading = false;
    }
  }

  function getStatusIcon(passed: boolean): string {
    return passed ? 'check' : 'x';
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'healthy': return 'status-healthy';
      case 'warning': return 'status-warning';
      case 'critical': return 'status-critical';
      default: return '';
    }
  }

  function getCategoryIcon(category: string): string {
    switch (category) {
      case 'Network': return 'network';
      case 'DNS': return 'dns';
      case 'Connectivity': return 'connectivity';
      case 'WiFi': return 'wifi';
      case 'Speed': return 'speed';
      default: return 'default';
    }
  }
</script>

<div class="page">
  <h1 class="title">Network Diagnostics</h1>

  <div class="card options-card">
    <h2 class="card-title">Test Options</h2>
    <div class="options-grid">
      <label class="option">
        <input type="checkbox" bind:checked={quickMode} />
        <span class="option-label">Quick Mode</span>
        <span class="option-desc">Run basic tests only (faster)</span>
      </label>
      <label class="option" class:disabled={quickMode}>
        <input type="checkbox" bind:checked={includeSpeed} disabled={quickMode} />
        <span class="option-label">Speed Test</span>
        <span class="option-desc">Include download speed test</span>
      </label>
      <label class="option">
        <input type="checkbox" bind:checked={includeWifi} />
        <span class="option-label">WiFi Analysis</span>
        <span class="option-desc">Check WiFi signal quality</span>
      </label>
    </div>
    <button class="btn btn-primary" onclick={runDiagnostics} disabled={loading}>
      {loading ? 'Running Diagnostics...' : 'Run Diagnostics'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if loading}
    <div class="card loading-card">
      <div class="spinner"></div>
      <p class="loading-text">{currentTest || 'Running diagnostics...'}</p>
      <p class="loading-hint">This may take a moment{!quickMode && includeSpeed ? ' (speed test in progress)' : ''}...</p>
    </div>
  {/if}

  {#if result && !loading}
    <div class="results-section">
      <!-- Summary Card -->
      <div class="card summary-card {getStatusClass(result.summary.overall_status)}">
        <div class="summary-header">
          <div class="summary-status">
            <span class="status-indicator {result.summary.overall_status}"></span>
            <span class="status-text">
              {#if result.summary.overall_status === 'healthy'}
                Network Healthy
              {:else if result.summary.overall_status === 'warning'}
                Minor Issues Detected
              {:else}
                Issues Detected
              {/if}
            </span>
          </div>
          <div class="summary-stats">
            <span class="stat passed">{result.summary.passed} passed</span>
            {#if result.summary.failed > 0}
              <span class="stat failed">{result.summary.failed} failed</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Test Results -->
      <div class="card tests-card">
        <h2 class="card-title">Test Results</h2>
        <div class="tests-list">
          {#each result.tests as test}
            <div class="test-item" class:passed={test.passed} class:failed={!test.passed}>
              <div class="test-icon {getCategoryIcon(test.category)}">
                {#if test.passed}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"></polyline>
                  </svg>
                {:else}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                {/if}
              </div>
              <div class="test-content">
                <div class="test-header">
                  <span class="test-name">{test.name}</span>
                  <span class="test-category">{test.category}</span>
                </div>
                <p class="test-message">{test.message}</p>
                <span class="test-duration">{test.duration_ms}ms</span>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Issues and Recommendations -->
      {#if result.issues.length > 0 || result.recommendations.length > 0}
        <div class="card recommendations-card">
          {#if result.issues.length > 0}
            <div class="issues-section">
              <h3 class="section-title">Issues Found</h3>
              <ul class="issues-list">
                {#each result.issues as issue}
                  <li>{issue}</li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if result.recommendations.length > 0}
            <div class="recommendations-section">
              <h3 class="section-title">Recommendations</h3>
              <ul class="recommendations-list">
                {#each result.recommendations as rec}
                  <li>{rec}</li>
                {/each}
              </ul>
            </div>
          {/if}
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

  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .options-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .options-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .option {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .option:hover:not(.disabled) {
    background-color: var(--bg-secondary);
  }

  .option.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .option input {
    width: 18px;
    height: 18px;
    margin-bottom: 0.25rem;
  }

  .option-label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .option-desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
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

  .results-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .summary-card {
    border-left: 4px solid var(--border);
  }

  .summary-card.status-healthy {
    border-left-color: var(--success);
  }

  .summary-card.status-warning {
    border-left-color: var(--warning);
  }

  .summary-card.status-critical {
    border-left-color: var(--error);
  }

  .summary-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .summary-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .status-indicator.healthy {
    background-color: var(--success);
  }

  .status-indicator.warning {
    background-color: var(--warning);
  }

  .status-indicator.critical {
    background-color: var(--error);
  }

  .status-text {
    font-size: 1.25rem;
    font-weight: 600;
  }

  .summary-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .stat.passed {
    color: var(--success);
  }

  .stat.failed {
    color: var(--error);
  }

  .tests-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .test-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    border-left: 3px solid var(--border);
  }

  .test-item.passed {
    border-left-color: var(--success);
  }

  .test-item.failed {
    border-left-color: var(--error);
  }

  .test-icon {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    padding: 4px;
  }

  .test-item.passed .test-icon {
    background-color: rgba(16, 185, 129, 0.2);
    color: var(--success);
  }

  .test-item.failed .test-icon {
    background-color: rgba(239, 68, 68, 0.2);
    color: var(--error);
  }

  .test-icon svg {
    width: 16px;
    height: 16px;
  }

  .test-content {
    flex: 1;
    min-width: 0;
  }

  .test-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .test-name {
    font-weight: 600;
  }

  .test-category {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .test-message {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .test-duration {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .recommendations-card {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section-title {
    margin: 0 0 0.75rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .issues-list {
    margin: 0;
    padding-left: 1.25rem;
    color: var(--error);
  }

  .issues-list li {
    margin-bottom: 0.25rem;
  }

  .recommendations-list {
    margin: 0;
    padding-left: 1.25rem;
  }

  .recommendations-list li {
    margin-bottom: 0.5rem;
  }
</style>
