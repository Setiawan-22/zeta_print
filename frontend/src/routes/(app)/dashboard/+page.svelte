<script lang="ts">
  import { onMount } from 'svelte';

  // State variables for real-time data
  let activePrinters = $state(0);
  let totalPrinters = $state(0);
  let jobsInQueue = $state(0);
  let inkAlerts = $state(0); // TODO: fetch actual ink alerts
  let uptime = $state("14d 2h");

  let printers = $state<any[]>([]);
  let logs = $state<any[]>([]);

  $effect(() => {
      // Fetch Stats
      fetch('/api/v1/dashboard/stats')
          .then(r => r.json())
          .then(res => {
              if (res.status === 'success') {
                  totalPrinters = res.data.total_printers;
                  activePrinters = res.data.total_printers; // Mocking all active for now
                  jobsInQueue = res.data.total_jobs;
              }
          })
          .catch(console.error);

      // Fetch Printers
      fetch('/api/v1/dashboard/printers')
          .then(r => r.json())
          .then(res => {
              if (res.status === 'success') {
                  printers = res.data.map((p: any) => ({
                      status: p.is_legacy ? 'blue' : 'green',
                      name: p.name,
                      computer: p.ip_address,
                      file: p.is_legacy ? 'PAPPL Legacy Mode' : 'Modern IPP Mode',
                      pages: `Rp ${p.cost_per_page_bw}`
                  }));
              }
          })
          .catch(console.error);

      // Fetch Print Queue for Logs
      fetch('/api/v1/dashboard/queue')
          .then(r => r.json())
          .then(res => {
              if (res.status === 'success') {
                  logs = res.data.map((j: any) => {
                      // Format date
                      const timeStr = new Date(j.created_at).toLocaleTimeString();
                      return {
                          time: timeStr !== 'Invalid Date' ? timeStr : j.created_at.split('T')[1]?.substring(0, 8) || j.created_at,
                          msg: `[${j.status}] ${j.document_name} (${j.user_id})`,
                          type: j.status === 'Failed' ? 'error' : (j.status === 'Processing' ? 'warning' : 'normal')
                      };
                  });
              }
          })
          .catch(console.error);
  });
</script>

<div class="dashboard">
  <!-- Metrics Grid -->
  <div class="metrics-grid">
      <div class="panel metric-card">
          <div class="metric-header text-muted text-sm font-mono">
              <span>ACTIVE PRINTERS</span>
              <span>🖨️</span>
          </div>
          <div class="metric-value">
              <span class="text-2xl font-bold">{activePrinters}</span>
              <span class="text-secondary">/{totalPrinters}</span>
          </div>
      </div>

      <div class="panel metric-card">
          <div class="metric-header text-muted text-sm font-mono">
              <span>JOBS IN QUEUE</span>
              <span>📄</span>
          </div>
          <div class="metric-value">
              <span class="text-2xl font-bold">{jobsInQueue}</span>
              <span class="badge badge-gray">+12/hr</span>
          </div>
      </div>

      <div class="panel metric-card">
          <div class="metric-header text-muted text-sm font-mono">
              <span>INK/TONER ALERTS</span>
              <span class="text-blue">⚠️</span>
          </div>
          <div class="metric-value">
              <span class="text-2xl font-bold text-blue">{inkAlerts}</span>
              <span class="text-secondary text-base">Low</span>
          </div>
      </div>

      <div class="panel metric-card">
          <div class="metric-header text-muted text-sm font-mono">
              <span>SYSTEM UPTIME</span>
              <span>⏱️</span>
          </div>
          <div class="metric-value">
              <span class="text-2xl font-bold">{uptime}</span>
          </div>
      </div>
  </div>

  <div class="content-grid">
      <div class="main-column">
          <!-- System Health -->
          <div class="panel">
              <div class="panel-header text-muted text-sm font-mono mb-4 flex justify-between">
                  <span>SYSTEM HEALTH</span>
                  <span>🔄</span>
              </div>
              <div class="health-grid">
                  <div class="health-item">
                      <span class="status-dot dot-green"></span>
                      <div class="health-text">
                          <div class="text-secondary text-sm">Single Binary</div>
                          <div class="font-bold">Running</div>
                      </div>
                  </div>
                  <div class="health-item border-left">
                      <span class="status-dot dot-green"></span>
                      <div class="health-text">
                          <div class="text-secondary text-sm">SQLite DB</div>
                          <div class="font-bold">Healthy (12ms)</div>
                      </div>
                  </div>
                  <div class="health-item border-left">
                      <span class="status-dot dot-green"></span>
                      <div class="health-text">
                          <div class="text-secondary text-sm">PAPPL Bridge</div>
                          <div class="font-bold">Connected</div>
                      </div>
                  </div>
              </div>
          </div>

          <!-- Real-Time Printer Status -->
          <div class="panel mt-4">
              <div class="panel-header flex justify-between items-center mb-4">
                  <span class="text-muted text-sm font-mono">REAL-TIME PRINTER STATUS</span>
              </div>
              <table class="printer-table">
                  <thead>
                      <tr>
                          <th>Status</th>
                          <th>Printer Name</th>
                          <th>Computer</th>
                          <th>File Name</th>
                          <th>Pages</th>
                      </tr>
                  </thead>
                  <tbody>
                      {#each printers as p}
                      <tr>
                          <td><span class="status-dot dot-{p.status}"></span></td>
                          <td class="font-bold">{p.name}</td>
                          <td class="text-secondary font-mono">{p.computer}</td>
                          <td class="{p.status === 'blue' ? 'text-blue' : (p.status === 'red' ? 'text-red' : '')}">
                              {p.file}
                          </td>
                          <td class="text-right font-mono {p.status === 'red' ? 'text-red' : ''}">{p.pages}</td>
                      </tr>
                      {/each}
                  </tbody>
              </table>
          </div>
      </div>

      <!-- Sidebar -->
      <div class="sidebar-column">
          <div class="panel activity-panel">
              <div class="panel-header text-muted text-sm font-mono mb-4 flex justify-between">
                  <span>ACTIVITY LOGS</span>
                  <span>=</span>
              </div>
              <div class="logs-container font-mono text-sm">
                  {#each logs as log}
                      <div class="log-entry {log.type}">
                          <span class="log-time text-secondary">{log.time}</span>
                          <span class="log-msg">{log.msg}</span>
                      </div>
                  {/each}
              </div>
              <div class="view-all">
                  <a href="/dashboard/logs" class="text-blue font-mono text-sm font-bold">VIEW ALL LOGS</a>
              </div>
          </div>
      </div>
  </div>
</div>

<style>
  .dashboard {
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .metrics-grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 16px;
  }

  .metric-card {
      display: flex;
      flex-direction: column;
      gap: 12px;
  }

  .metric-header {
      display: flex;
      justify-content: space-between;
      letter-spacing: 0.05em;
  }

  .metric-value {
      display: flex;
      align-items: baseline;
      gap: 4px;
  }

  .badge-gray {
      background-color: var(--border-color);
      color: var(--text-secondary);
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 0.75rem;
      margin-left: 8px;
  }

  .content-grid {
      display: grid;
      grid-template-columns: 1fr 350px;
      gap: 16px;
  }

  .main-column {
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .health-grid {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
  }

  .health-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 8px 16px;
  }

  .border-left {
      border-left: 1px solid var(--border-color);
  }

  .mt-4 { margin-top: 16px; }
  .mb-4 { margin-bottom: 16px; }

  /* Table Styles */
  .printer-table {
      width: 100%;
      border-collapse: collapse;
  }

  .printer-table th {
      text-align: left;
      color: var(--text-muted);
      font-weight: 500;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.875rem;
      padding: 12px 0;
      border-bottom: 1px solid var(--border-color);
  }

  .printer-table td {
      padding: 16px 0;
      border-bottom: 1px solid var(--border-color);
  }

  .printer-table tbody tr:last-child td {
      border-bottom: none;
  }

  .printer-table th:last-child,
  .printer-table td:last-child {
      text-align: right;
  }

  /* Logs Sidebar */
  .activity-panel {
      display: flex;
      flex-direction: column;
      height: 100%;
  }

  .logs-container {
      display: flex;
      flex-direction: column;
      gap: 8px;
      flex: 1;
  }

  .log-entry {
      display: flex;
      flex-direction: column;
      padding: 6px 8px;
      border-radius: 4px;
      line-height: 1.4;
  }

  .log-entry.error {
      background-color: var(--status-red-bg);
      color: var(--status-red);
  }

  .log-entry.warning {
      background-color: var(--accent-blue-bg);
      color: var(--accent-blue);
  }

  .log-time {
      margin-right: 8px;
  }

  .view-all {
      margin-top: 16px;
      padding-top: 16px;
      border-top: 1px solid var(--border-color);
      text-align: center;
  }
</style>
