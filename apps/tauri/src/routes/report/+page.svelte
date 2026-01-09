<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface GeneratedReport {
    content: string;
    mime_type: string;
    file_extension: string;
    is_binary: boolean;
    health_score: number;
    health_status: string;
  }

  type ReportFormat = 'json' | 'text' | 'markdown' | 'html';

  let selectedFormat = $state<ReportFormat>('html');
  let includeRawData = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let report = $state<GeneratedReport | null>(null);
  let previewMode = $state<'rendered' | 'source'>('rendered');

  const formats: { value: ReportFormat; label: string; description: string }[] = [
    { value: 'html', label: 'HTML', description: 'Rich formatted report viewable in browser' },
    { value: 'markdown', label: 'Markdown', description: 'Plain text with formatting for documentation' },
    { value: 'json', label: 'JSON', description: 'Machine-readable structured data' },
    { value: 'text', label: 'Plain Text', description: 'Simple text format for logs' },
  ];

  async function generateReport() {
    loading = true;
    error = null;
    report = null;

    try {
      report = await invoke<GeneratedReport>('generate_report', {
        format: selectedFormat,
        includeRawData,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function getHealthColor(status: string): string {
    switch (status) {
      case 'good': return 'var(--success)';
      case 'warning': return 'var(--warning)';
      case 'critical': return 'var(--error)';
      default: return 'var(--text-secondary)';
    }
  }

  function downloadReport() {
    if (!report) return;

    const blob = new Blob([report.content], { type: report.mime_type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `netdiag-report-${new Date().toISOString().slice(0, 10)}.${report.file_extension}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function copyToClipboard() {
    if (!report) return;
    await navigator.clipboard.writeText(report.content);
  }
</script>

<div class="page">
  <h1 class="title">Generate Report</h1>

  <div class="card options-card">
    <h2 class="card-title">Report Options</h2>

    <div class="format-section">
      <label class="section-label">Format</label>
      <div class="format-grid">
        {#each formats as fmt}
          <button
            class="format-option"
            class:selected={selectedFormat === fmt.value}
            onclick={() => selectedFormat = fmt.value}
          >
            <span class="format-label">{fmt.label}</span>
            <span class="format-desc">{fmt.description}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="options-row">
      <label class="checkbox-option">
        <input type="checkbox" bind:checked={includeRawData} />
        <span>Include raw diagnostic data</span>
      </label>
    </div>

    <button class="btn btn-primary" onclick={generateReport} disabled={loading}>
      {loading ? 'Generating Report...' : 'Generate Report'}
    </button>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if loading}
    <div class="card loading-card">
      <div class="spinner"></div>
      <p class="loading-text">Collecting diagnostic data...</p>
      <p class="loading-hint">Running network tests and generating report</p>
    </div>
  {/if}

  {#if report && !loading}
    <div class="results-section">
      <!-- Health Summary -->
      <div class="card health-card">
        <div class="health-header">
          <div class="health-score-container">
            <div
              class="health-score"
              style="--score-color: {getHealthColor(report.health_status)}"
            >
              {report.health_score}
            </div>
            <span class="health-label">Health Score</span>
          </div>
          <div class="health-status" style="color: {getHealthColor(report.health_status)}">
            {report.health_status.charAt(0).toUpperCase() + report.health_status.slice(1)}
          </div>
        </div>
      </div>

      <!-- Report Actions -->
      <div class="card actions-card">
        <div class="actions-row">
          <button class="btn btn-secondary" onclick={downloadReport}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            Download {report.file_extension.toUpperCase()}
          </button>
          <button class="btn btn-secondary" onclick={copyToClipboard}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            Copy to Clipboard
          </button>
        </div>
      </div>

      <!-- Report Preview -->
      <div class="card preview-card">
        <div class="preview-header">
          <h2 class="card-title">Report Preview</h2>
          {#if selectedFormat === 'html' || selectedFormat === 'markdown'}
            <div class="preview-toggle">
              <button
                class="toggle-btn"
                class:active={previewMode === 'rendered'}
                onclick={() => previewMode = 'rendered'}
              >
                Rendered
              </button>
              <button
                class="toggle-btn"
                class:active={previewMode === 'source'}
                onclick={() => previewMode = 'source'}
              >
                Source
              </button>
            </div>
          {/if}
        </div>

        <div class="preview-content">
          {#if selectedFormat === 'html' && previewMode === 'rendered'}
            <iframe
              class="html-preview"
              srcdoc={report.content}
              sandbox="allow-same-origin"
              title="Report Preview"
            ></iframe>
          {:else if selectedFormat === 'markdown' && previewMode === 'rendered'}
            <div class="markdown-preview">
              <pre class="source-code">{report.content}</pre>
            </div>
          {:else}
            <pre class="source-code">{report.content}</pre>
          {/if}
        </div>
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

  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .options-card {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section-label {
    display: block;
    font-weight: 600;
    font-size: 0.875rem;
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 0.75rem;
  }

  .format-option {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem 1rem;
    background-color: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: 0.5rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .format-option:hover {
    background-color: var(--bg-secondary);
  }

  .format-option.selected {
    border-color: var(--accent);
    background-color: rgba(59, 130, 246, 0.1);
  }

  .format-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .format-desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .options-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
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

  .health-card {
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
  }

  .health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .health-score-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .health-score {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--score-color);
    border: 3px solid var(--score-color);
    background-color: var(--bg-primary);
  }

  .health-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .health-status {
    font-size: 1.5rem;
    font-weight: 600;
  }

  .actions-card {
    padding: 1rem;
  }

  .actions-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .btn-icon {
    width: 18px;
    height: 18px;
    margin-right: 0.5rem;
  }

  .preview-card {
    display: flex;
    flex-direction: column;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .preview-header .card-title {
    margin: 0;
  }

  .preview-toggle {
    display: flex;
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    padding: 0.25rem;
  }

  .toggle-btn {
    padding: 0.375rem 0.75rem;
    border: none;
    background: none;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    transition: all 0.15s ease;
  }

  .toggle-btn:hover {
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background-color: var(--bg-primary);
    color: var(--text-primary);
  }

  .preview-content {
    background-color: var(--bg-tertiary);
    border-radius: 0.375rem;
    overflow: hidden;
  }

  .html-preview {
    width: 100%;
    height: 500px;
    border: none;
    background-color: white;
  }

  .markdown-preview {
    padding: 1rem;
    overflow-x: auto;
  }

  .source-code {
    margin: 0;
    padding: 1rem;
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    font-size: 0.75rem;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 500px;
    overflow-y: auto;
  }

  /* Mobile adjustments */
  @media (max-width: 768px) {
    .format-grid {
      grid-template-columns: 1fr;
    }

    .health-header {
      flex-direction: column;
      text-align: center;
    }

    .actions-row {
      flex-direction: column;
    }

    .actions-row .btn {
      width: 100%;
    }
  }
</style>
