<script lang="ts">
  import { onMount } from 'svelte';
  import { eventManager } from '$lib/stores/events.svelte';
  import { Plus, Edit2, Trash2, Printer as PrinterIcon } from '@lucide/svelte';

  interface Printer {
      id: string;
      name: string;
      ip_address: string;
      is_legacy: boolean;
      cost_per_page_bw: number;
      cost_per_page_color: number;
      is_restricted: boolean;
  }

  let printers = $state<Printer[]>([]);
  let isModalOpen = $state(false);
  let isEditing = $state(false);
  let currentPrinterId = $state('');

  // Form State
  let formName = $state('');
  let formIp = $state('');
  let formLegacy = $state(false);
  let formCostBw = $state(0);
  let formCostColor = $state(0);
  let formRestricted = $state(false);

  function resetForm() {
      formName = '';
      formIp = '';
      formLegacy = false;
      formCostBw = 0;
      formCostColor = 0;
      formRestricted = false;
      isEditing = false;
      currentPrinterId = '';
  }

  function fetchPrinters() {
      fetch('/api/v1/dashboard/printers')
          .then(r => r.json())
          .then(res => {
              if (res.status === 'success') {
                  printers = res.data;
              }
          })
          .catch(e => console.error(e));
  }

  $effect(() => {
      fetchPrinters();
  });

  function openAddModal() {
      resetForm();
      isModalOpen = true;
  }

  function openEditModal(printer: Printer) {
      formName = printer.name;
      formIp = printer.ip_address;
      formLegacy = printer.is_legacy;
      formCostBw = printer.cost_per_page_bw;
      formCostColor = printer.cost_per_page_color;
      formRestricted = printer.is_restricted;
      currentPrinterId = printer.id;
      isEditing = true;
      isModalOpen = true;
  }

  async function savePrinter() {
      const payload = {
          name: formName,
          ip_address: formIp,
          is_legacy: formLegacy,
          cost_per_page_bw: Number(formCostBw),
          cost_per_page_color: Number(formCostColor),
          is_restricted: formRestricted
      };

      try {
          const url = isEditing 
              ? `/api/v1/dashboard/printers/${currentPrinterId}` 
              : `/api/v1/dashboard/printers`;
          
          const res = await fetch(url, {
              method: isEditing ? 'PUT' : 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(payload)
          });
          const data = await res.json();
          
          if (data.status === 'success') {
              eventManager.addEvent({
                  id: crypto.randomUUID(),
                  event_type: 'success',
                  message: `Printer successfully ${isEditing ? 'updated' : 'added'}!`,
                  timestamp: new Date().toISOString()
              });
              isModalOpen = false;
              fetchPrinters();
          } else {
              throw new Error(data.message || 'Unknown error');
          }
      } catch (err: any) {
          eventManager.addEvent({
              id: crypto.randomUUID(),
              event_type: 'error',
              message: `Failed to save printer: ${err.message}`,
              timestamp: new Date().toISOString()
          });
      }
  }

  async function deletePrinter(id: string) {
      if (!confirm('Are you sure you want to delete this printer?')) return;
      try {
          const res = await fetch(`/api/v1/dashboard/printers/${id}`, {
              method: 'DELETE'
          });
          const data = await res.json();
          if (data.status === 'success') {
              eventManager.addEvent({
                  id: crypto.randomUUID(),
                  event_type: 'success',
                  message: 'Printer deleted successfully!',
                  timestamp: new Date().toISOString()
              });
              fetchPrinters();
          } else {
              throw new Error(data.message);
          }
      } catch (err: any) {
          eventManager.addEvent({
              id: crypto.randomUUID(),
              event_type: 'error',
              message: `Failed to delete printer: ${err.message}`,
              timestamp: new Date().toISOString()
          });
      }
  }
</script>

<div class="printers-page">
  <div class="page-header flex justify-between items-center mb-6">
      <div>
          <h1 class="text-2xl font-bold">Printer Management</h1>
          <p class="text-secondary text-sm font-mono mt-1">Configure physical and virtual printers</p>
      </div>
      <button class="btn btn-primary" onclick={openAddModal}>
          <Plus size={16} /> Add Printer
      </button>
  </div>

  <div class="panel">
      <table class="printer-table">
          <thead>
              <tr>
                  <th>Name</th>
                  <th>IP Address</th>
                  <th>Type</th>
                  <th>BW Cost</th>
                  <th>Color Cost</th>
                  <th>Access</th>
                  <th class="text-right">Actions</th>
              </tr>
          </thead>
          <tbody>
              {#if printers.length === 0}
                  <tr>
                      <td colspan="7" class="text-center text-secondary py-8">No printers configured yet.</td>
                  </tr>
              {/if}
              {#each printers as p}
                  <tr>
                      <td class="font-bold flex items-center gap-2">
                          <PrinterIcon size={16} class="text-blue" />
                          {p.name}
                      </td>
                      <td class="font-mono text-secondary">{p.ip_address}</td>
                      <td>
                          {#if p.is_legacy}
                              <span class="badge badge-legacy">Legacy (PAPPL)</span>
                          {:else}
                              <span class="badge badge-modern">Modern (IPP)</span>
                          {/if}
                      </td>
                      <td class="font-mono text-secondary">Rp {p.cost_per_page_bw}</td>
                      <td class="font-mono text-secondary">Rp {p.cost_per_page_color}</td>
                      <td>
                          {#if p.is_restricted}
                              <span class="badge badge-restricted">Restricted</span>
                          {:else}
                              <span class="badge badge-public">Public</span>
                          {/if}
                      </td>
                      <td class="text-right">
                          <button class="action-btn text-blue" onclick={() => openEditModal(p)} title="Edit">
                              <Edit2 size={16} />
                          </button>
                          <button class="action-btn text-red ml-2" onclick={() => deletePrinter(p.id)} title="Delete">
                              <Trash2 size={16} />
                          </button>
                      </td>
                  </tr>
              {/each}
          </tbody>
      </table>
  </div>
</div>

{#if isModalOpen}
  <div class="modal-backdrop">
      <div class="modal-content panel">
          <div class="modal-header flex justify-between items-center mb-6">
              <h2 class="text-xl font-bold">{isEditing ? 'Edit Printer' : 'Add New Printer'}</h2>
              <button class="icon-btn" onclick={() => isModalOpen = false}>✕</button>
          </div>
          
          <div class="form-grid">
              <div class="form-group">
                  <label for="name">Printer Name</label>
                  <input id="name" type="text" class="input" bind:value={formName} placeholder="e.g. HR-Color-1" />
              </div>
              <div class="form-group">
                  <label for="ip">IP Address</label>
                  <input id="ip" type="text" class="input font-mono" bind:value={formIp} placeholder="192.168.1.50" />
              </div>
              
              <div class="form-group">
                  <label for="bw">BW Cost (Rp)</label>
                  <input id="bw" type="number" class="input font-mono" bind:value={formCostBw} />
              </div>
              <div class="form-group">
                  <label for="color">Color Cost (Rp)</label>
                  <input id="color" type="number" class="input font-mono" bind:value={formCostColor} />
              </div>

              <div class="form-group checkbox-group">
                  <label class="flex items-center gap-2 cursor-pointer">
                      <input type="checkbox" bind:checked={formLegacy} />
                      <span class="text-sm">Legacy Printer (Requires PAPPL Bridge)</span>
                  </label>
              </div>

              <div class="form-group checkbox-group">
                  <label class="flex items-center gap-2 cursor-pointer">
                      <input type="checkbox" bind:checked={formRestricted} />
                      <span class="text-sm">Restricted Access (Requires ACL)</span>
                  </label>
              </div>
          </div>

          <div class="modal-footer mt-8 flex justify-end gap-4">
              <button class="btn btn-secondary" onclick={() => isModalOpen = false}>Cancel</button>
              <button class="btn btn-primary" onclick={savePrinter}>
                  {isEditing ? 'Save Changes' : 'Add Printer'}
              </button>
          </div>
      </div>
  </div>
{/if}

<style>
  .printers-page {
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .mb-6 { margin-bottom: 24px; }
  .mt-1 { margin-top: 4px; }
  .mt-8 { margin-top: 32px; }
  .ml-2 { margin-left: 8px; }

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
      padding: 12px 16px;
      border-bottom: 1px solid var(--border-color);
  }

  .printer-table td {
      padding: 16px;
      border-bottom: 1px solid var(--border-color);
  }

  .printer-table tbody tr:last-child td {
      border-bottom: none;
  }

  .printer-table tbody tr:hover {
      background-color: var(--bg-panel-hover);
  }

  .badge {
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
  }

  .badge-legacy { background-color: var(--status-yellow-bg); color: var(--status-yellow); }
  .badge-modern { background-color: var(--accent-blue-bg); color: var(--accent-blue); }
  .badge-restricted { background-color: var(--status-red-bg); color: var(--status-red); }
  .badge-public { background-color: var(--status-green-bg); color: var(--status-green); }

  .action-btn {
      background: none;
      border: none;
      cursor: pointer;
      padding: 6px;
      border-radius: 4px;
      transition: background-color 0.2s;
  }

  .action-btn:hover {
      background-color: var(--bg-app);
  }

  /* Modal Styles */
  .modal-backdrop {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background-color: rgba(0,0,0,0.6);
      backdrop-filter: blur(4px);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 100;
  }

  .modal-content {
      width: 100%;
      max-width: 500px;
      box-shadow: 0 24px 48px rgba(0,0,0,0.4);
      border: 1px solid var(--border-color);
  }

  .form-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 16px;
  }

  .form-group {
      display: flex;
      flex-direction: column;
      gap: 8px;
  }

  .form-group:nth-child(1),
  .form-group:nth-child(2) {
      grid-column: span 2;
  }

  .checkbox-group {
      grid-column: span 2;
      flex-direction: row;
      align-items: center;
      margin-top: 8px;
  }

  label {
      font-size: 0.875rem;
      color: var(--text-secondary);
  }

  .input {
      background-color: var(--bg-app);
      border: 1px solid var(--border-color);
      color: var(--text-primary);
      padding: 10px 12px;
      border-radius: 6px;
      outline: none;
      transition: border-color 0.2s;
  }

  .input:focus {
      border-color: var(--accent-blue);
  }

  input[type="checkbox"] {
      width: 16px;
      height: 16px;
      accent-color: var(--accent-blue);
  }

  .btn {
      padding: 8px 16px;
      border-radius: 6px;
      font-weight: 600;
      font-size: 0.875rem;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 8px;
      border: none;
      transition: opacity 0.2s;
  }

  .btn:hover { opacity: 0.9; }
  
  .btn-primary {
      background-color: var(--accent-blue);
      color: #fff;
  }

  .btn-secondary {
      background-color: transparent;
      color: var(--text-secondary);
      border: 1px solid var(--border-color);
  }

  .btn-secondary:hover {
      background-color: var(--bg-panel-hover);
      color: var(--text-primary);
  }
</style>
