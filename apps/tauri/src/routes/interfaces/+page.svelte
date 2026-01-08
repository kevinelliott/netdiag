<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface InterfaceInfo {
    name: string;
    friendly_name?: string;
    mac_address?: string;
    ipv4_addresses: string[];
    ipv6_addresses: string[];
    is_up: boolean;
    is_loopback: boolean;
    is_default: boolean;
    interface_type: string;
  }

  let interfaces = $state<InterfaceInfo[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showLoopback = $state(false);

  async function loadInterfaces() {
    loading = true;
    error = null;
    try {
      interfaces = await invoke<InterfaceInfo[]>('get_interfaces');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadInterfaces();
  });

  let filteredInterfaces = $derived(
    showLoopback ? interfaces : interfaces.filter(i => !i.is_loopback)
  );
</script>

<div class="page">
  <div class="header-row">
    <h1 class="title">Network Interfaces</h1>
    <div class="controls">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={showLoopback} />
        Show loopback
      </label>
      <button class="btn btn-secondary" onclick={loadInterfaces}>
        {loading ? 'Refreshing...' : 'Refresh'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">Error: {error}</div>
  {/if}

  {#if loading && interfaces.length === 0}
    <p class="loading-text">Loading interfaces...</p>
  {:else}
    <div class="interface-list">
      {#each filteredInterfaces as iface}
        <div class="card interface-card" class:is-default={iface.is_default}>
          <div class="interface-header">
            <div class="interface-name">
              <span class="name">{iface.friendly_name || iface.name}</span>
              {#if iface.name !== iface.friendly_name}
                <span class="tech-name">({iface.name})</span>
              {/if}
              {#if iface.is_default}
                <span class="badge default">Default</span>
              {/if}
            </div>
            <div class="interface-status">
              <span class="status-badge" class:up={iface.is_up} class:down={!iface.is_up}>
                {iface.is_up ? 'Up' : 'Down'}
              </span>
              <span class="type-badge">{iface.interface_type}</span>
            </div>
          </div>

          <div class="interface-details">
            {#if iface.mac_address}
              <div class="detail">
                <span class="label">MAC Address</span>
                <span class="value mono">{iface.mac_address}</span>
              </div>
            {/if}

            {#if iface.ipv4_addresses.length > 0}
              <div class="detail">
                <span class="label">IPv4 Addresses</span>
                <div class="addresses">
                  {#each iface.ipv4_addresses as addr}
                    <span class="addr mono">{addr}</span>
                  {/each}
                </div>
              </div>
            {/if}

            {#if iface.ipv6_addresses.length > 0}
              <div class="detail">
                <span class="label">IPv6 Addresses</span>
                <div class="addresses">
                  {#each iface.ipv6_addresses as addr}
                    <span class="addr mono ipv6">{addr}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      {/each}
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
    flex-wrap: wrap;
    gap: 1rem;
  }

  .title {
    margin: 0;
    font-size: 1.5rem;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    color: var(--error);
  }

  .loading-text {
    color: var(--text-secondary);
    font-style: italic;
  }

  .interface-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .interface-card {
    transition: border-color 0.15s ease;
  }

  .interface-card.is-default {
    border-color: var(--accent);
  }

  .interface-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .interface-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .name {
    font-weight: 600;
    font-size: 1.125rem;
  }

  .tech-name {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .badge {
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .badge.default {
    background-color: var(--accent);
    color: white;
  }

  .interface-status {
    display: flex;
    gap: 0.5rem;
  }

  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .status-badge.up {
    background-color: rgba(16, 185, 129, 0.1);
    color: var(--success);
  }

  .status-badge.down {
    background-color: rgba(239, 68, 68, 0.1);
    color: var(--error);
  }

  .type-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .interface-details {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .detail {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .value {
    font-weight: 500;
  }

  .mono {
    font-family: ui-monospace, monospace;
    font-size: 0.875rem;
  }

  .addresses {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .addr {
    color: var(--text-primary);
  }

  .addr.ipv6 {
    font-size: 0.75rem;
    word-break: break-all;
  }
</style>
