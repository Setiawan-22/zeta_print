<script lang="ts">
  import { eventManager } from '$lib/stores/events.svelte';
  import { FileTerminal, Info, AlertTriangle, CheckCircle, XCircle } from '@lucide/svelte';
</script>

<div class="logs-page">
  <div class="page-header flex justify-between items-center mb-6">
      <div>
          <h1 class="text-2xl font-bold">System Logs</h1>
          <p class="text-secondary text-sm font-mono mt-1">Real-time event logs and system notifications</p>
      </div>
  </div>

  <div class="panel">
      <div class="logs-container">
          {#if eventManager.history.length === 0}
              <div class="text-center text-secondary py-8 font-mono">
                  No logs recorded in the current session.
              </div>
          {/if}
          
          {#each eventManager.history as log (log.id)}
              <div class="log-entry">
                  <div class="log-time font-mono text-xs text-secondary">
                      {new Date(log.timestamp).toLocaleTimeString()}
                  </div>
                  <div class="log-icon">
                      {#if log.event_type === 'success'}
                          <CheckCircle size={16} class="text-green" />
                      {:else if log.event_type === 'error'}
                          <XCircle size={16} class="text-red" />
                      {:else if log.event_type === 'warning'}
                          <AlertTriangle size={16} class="text-yellow" />
                      {:else}
                          <Info size={16} class="text-blue" />
                      {/if}
                  </div>
                  <div class="log-message font-mono text-sm">
                      <span class="log-type">[{log.event_type.toUpperCase()}]</span> {log.message}
                  </div>
              </div>
          {/each}
      </div>
  </div>
</div>

<style>
  .logs-page {
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .mb-6 { margin-bottom: 24px; }
  .mt-1 { margin-top: 4px; }
  .py-8 { padding-top: 32px; padding-bottom: 32px; }

  .logs-container {
      display: flex;
      flex-direction: column;
      gap: 8px;
  }

  .log-entry {
      display: flex;
      align-items: center;
      gap: 16px;
      padding: 12px;
      background-color: var(--bg-app);
      border: 1px solid var(--border-color);
      border-radius: 6px;
      transition: background-color 0.2s;
  }

  .log-entry:hover {
      background-color: var(--bg-panel-hover);
  }

  .log-time {
      width: 80px;
      flex-shrink: 0;
  }

  .log-icon {
      display: flex;
      align-items: center;
      justify-content: center;
  }

  .log-message {
      color: var(--text-primary);
  }

  .log-type {
      color: var(--text-secondary);
      font-weight: bold;
      margin-right: 8px;
  }
  
  .text-yellow { color: var(--status-yellow); }
</style>
