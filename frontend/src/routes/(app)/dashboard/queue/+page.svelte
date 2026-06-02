<script lang="ts">
  import { onMount } from 'svelte';
  import { ListOrdered, CheckCircle, XCircle, Clock } from '@lucide/svelte';

  interface PrintJob {
      id: string;
      user_id: string;
      printer_id: string;
      document_name: string;
      total_pages: number;
      is_color: boolean;
      total_cost: number;
      status: string;
      created_at: string;
  }

  let queue = $state<PrintJob[]>([]);

  function fetchQueue() {
      fetch('/api/v1/dashboard/queue')
          .then(r => r.json())
          .then(res => {
              if (res.status === 'success') {
                  queue = res.data;
              }
          })
          .catch(e => console.error(e));
  }

  $effect(() => {
      fetchQueue();
      const interval = setInterval(fetchQueue, 10000); // Auto refresh every 10s
      return () => clearInterval(interval);
  });
</script>

<div class="queue-page">
  <div class="page-header flex justify-between items-center mb-6">
      <div>
          <h1 class="text-2xl font-bold">Print Queue</h1>
          <p class="text-secondary text-sm font-mono mt-1">Real-time status of all printing jobs</p>
      </div>
  </div>

  <div class="panel">
      <table class="queue-table">
          <thead>
              <tr>
                  <th>Time</th>
                  <th>Document</th>
                  <th>User</th>
                  <th>Pages</th>
                  <th>Type</th>
                  <th>Cost</th>
                  <th>Status</th>
              </tr>
          </thead>
          <tbody>
              {#if queue.length === 0}
                  <tr>
                      <td colspan="7" class="text-center text-secondary py-8">Queue is currently empty.</td>
                  </tr>
              {/if}
              {#each queue as job}
                  <tr>
                      <td class="font-mono text-secondary text-sm">{new Date(job.created_at).toLocaleString()}</td>
                      <td class="font-bold flex items-center gap-2">
                          <ListOrdered size={16} class="text-blue" />
                          {job.document_name}
                      </td>
                      <td class="text-secondary">{job.user_id}</td>
                      <td class="font-mono">{job.total_pages}</td>
                      <td>
                          {#if job.is_color}
                              <span class="badge badge-color">Color</span>
                          {:else}
                              <span class="badge badge-bw">B&W</span>
                          {/if}
                      </td>
                      <td class="font-mono text-secondary">Rp {job.total_cost}</td>
                      <td>
                          {#if job.status === 'Completed'}
                              <span class="badge badge-success"><CheckCircle size={12} class="inline" /> {job.status}</span>
                          {:else if job.status === 'Failed' || job.status === 'Denied'}
                              <span class="badge badge-error"><XCircle size={12} class="inline" /> {job.status}</span>
                          {:else}
                              <span class="badge badge-processing"><Clock size={12} class="inline" /> {job.status}</span>
                          {/if}
                      </td>
                  </tr>
              {/each}
          </tbody>
      </table>
  </div>
</div>

<style>
  .queue-page {
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .mb-6 { margin-bottom: 24px; }
  .mt-1 { margin-top: 4px; }
  .py-8 { padding-top: 32px; padding-bottom: 32px; }

  .queue-table {
      width: 100%;
      border-collapse: collapse;
  }

  .queue-table th {
      text-align: left;
      color: var(--text-muted);
      font-weight: 500;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.875rem;
      padding: 12px 16px;
      border-bottom: 1px solid var(--border-color);
  }

  .queue-table td {
      padding: 16px;
      border-bottom: 1px solid var(--border-color);
  }

  .queue-table tbody tr:last-child td {
      border-bottom: none;
  }

  .queue-table tbody tr:hover {
      background-color: var(--bg-panel-hover);
  }

  .badge {
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
      display: inline-flex;
      align-items: center;
      gap: 4px;
  }

  .badge-color { background-color: var(--accent-blue-bg); color: var(--accent-blue); }
  .badge-bw { background-color: var(--border-color); color: var(--text-primary); }
  .badge-success { background-color: var(--status-green-bg); color: var(--status-green); }
  .badge-error { background-color: var(--status-red-bg); color: var(--status-red); }
  .badge-processing { background-color: var(--status-yellow-bg); color: var(--status-yellow); }
</style>
