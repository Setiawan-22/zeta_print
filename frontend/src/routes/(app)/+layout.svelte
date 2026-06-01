<script lang="ts">
  import { page } from '$app/stores';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { eventManager } from '$lib/stores/events.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
  import { LayoutDashboard, Printer, ListOrdered, FileTerminal, LogOut, Sun, Moon, Bell, ChevronLeft, ChevronRight } from '@lucide/svelte';

  let { children } = $props();

  $effect(() => {
      eventManager.connect();
      return () => eventManager.disconnect();
  });

  let isDarkMode = $state(true);
  let isSidebarCollapsed = $state(true);
  let profileOpen = $state(false);

  function logout() {
      auth.logout();
      goto('/login');
  }

  function toggleTheme() {
      isDarkMode = !isDarkMode;
      if (browser) {
          document.body.classList.toggle('light-theme', !isDarkMode);
      }
  }

  // Active link helper
  const isActive = (path: string) => $page.url.pathname.startsWith(path) ? 'active' : '';
</script>

<div class="layout-container">
  <!-- Full Width Top Header -->
  <header class="top-header">
      <div class="header-left">
          <div class="logo">
              <span class="logo-icon">🖨️</span>
              <span class="logo-text">ZetaPrint</span>
          </div>
      </div>

      <div class="header-right">
          <button class="icon-btn" onclick={toggleTheme}>
              {#if isDarkMode}<Sun size={20} />{:else}<Moon size={20} />{/if}
          </button>
          <button class="icon-btn"><Bell size={20} /></button>
          
          <div class="profile-container">
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="avatar" onclick={() => profileOpen = !profileOpen}>U</div>
              
              {#if profileOpen}
              <div class="profile-dropdown">
                  <button class="dropdown-item text-red" onclick={logout}>
                      <LogOut size={16} />
                      <span>Logout</span>
                  </button>
              </div>
              {/if}
          </div>
      </div>
  </header>

  <!-- Below Header Content -->
  <div class="content-wrapper">
      <!-- Sidebar -->
      <aside class="sidebar" class:collapsed={isSidebarCollapsed}>
          <button class="collapse-btn" onclick={() => isSidebarCollapsed = !isSidebarCollapsed}>
              {#if isSidebarCollapsed}<ChevronRight size={16} />{:else}<ChevronLeft size={16} />{/if}
          </button>
          <nav class="sidebar-nav">
             <a href="/dashboard" class="nav-link {isActive('/dashboard')}">
                <LayoutDashboard size={20} />
                {#if !isSidebarCollapsed}<span>Dashboard</span>{/if}
             </a>
             <a href="/dashboard/printers" class="nav-link {isActive('/dashboard/printers')}">
                <Printer size={20} />
                {#if !isSidebarCollapsed}<span>Printers</span>{/if}
             </a>
             <a href="/dashboard/queue" class="nav-link {isActive('/dashboard/queue')}">
                <ListOrdered size={20} />
                {#if !isSidebarCollapsed}<span>Print Queue</span>{/if}
             </a>
             <a href="/dashboard/logs" class="nav-link {isActive('/dashboard/logs')}">
                <FileTerminal size={20} />
                {#if !isSidebarCollapsed}<span>System Logs</span>{/if}
             </a>
          </nav>
      </aside>

      <!-- Main Content Area -->
      <main class="main-content">
          {@render children()}
      </main>
  </div>
</div>

<ToastContainer />

<style>
  :global(body) { margin: 0; }
  
  .layout-container {
      display: flex;
      flex-direction: column;
      height: 100vh;
      background-color: var(--bg-app);
      overflow: hidden;
  }

  .top-header {
      height: 64px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 24px;
      background-color: var(--bg-panel);
      border-bottom: 1px solid var(--border-color);
      flex-shrink: 0;
      z-index: 20;
  }

  .header-left {
      display: flex;
      align-items: center;
      gap: 32px;
  }

  .logo {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--text-primary);
  }

  .logo-icon { color: var(--accent-blue); }

  .header-right {
      display: flex;
      align-items: center;
      gap: 16px;
  }

  /* Profile Dropdown */
  .profile-container {
      position: relative;
  }

  .profile-dropdown {
      position: absolute;
      top: 100%;
      right: 0;
      margin-top: 8px;
      background-color: var(--bg-panel);
      border: 1px solid var(--border-color);
      border-radius: 6px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
      min-width: 150px;
      padding: 8px;
      z-index: 50;
  }

  .dropdown-item {
      display: flex;
      align-items: center;
      gap: 12px;
      width: 100%;
      padding: 10px 12px;
      background: none;
      border: none;
      border-radius: 4px;
      color: var(--text-primary);
      font-size: 0.875rem;
      font-weight: 500;
      cursor: pointer;
      text-align: left;
      transition: background-color 0.2s;
  }

  .dropdown-item:hover {
      background-color: var(--bg-panel-hover);
  }

  .dropdown-item.text-red {
      color: var(--status-red);
  }

  .dropdown-item.text-red:hover {
      background-color: var(--status-red-bg);
  }

  .content-wrapper {
      display: flex;
      flex: 1;
      overflow: hidden;
  }

  .sidebar {
      position: relative;
      width: 240px;
      background-color: var(--bg-panel);
      border-right: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
      transition: width 0.3s ease;
      flex-shrink: 0;
      height: 100%;
  }

  .sidebar.collapsed {
      width: 68px;
  }

  .collapse-btn {
      position: absolute;
      right: -12px;
      top: 50%;
      transform: translateY(-50%);
      width: 24px;
      height: 24px;
      border-radius: 12px;
      background-color: var(--bg-panel);
      border: 1px solid var(--border-color);
      color: var(--text-secondary);
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      z-index: 30;
      transition: all 0.2s;
      padding: 0;
  }

  .collapse-btn:hover {
      color: var(--text-primary);
      border-color: var(--accent-blue);
  }
  
  .sidebar-nav {
      flex: 1;
      padding: 16px 8px;
      display: flex;
      flex-direction: column;
      gap: 8px;
  }

  .nav-link {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 14px;
      border-radius: 6px;
      color: var(--text-secondary);
      text-decoration: none;
      font-weight: 500;
      font-size: 0.875rem;
      white-space: nowrap;
      transition: all 0.2s;
      border: none;
      background: none;
      width: 100%;
      cursor: pointer;
      text-align: left;
  }

  .sidebar.collapsed .nav-link {
      padding: 10px;
      justify-content: center;
  }

  .nav-link:hover {
      background-color: var(--bg-panel-hover);
      color: var(--text-primary);
  }

  .nav-link.active {
      background-color: var(--accent-blue-bg);
      color: var(--accent-blue);
  }
  
  .main-content {
      flex: 1;
      overflow-y: auto;
      padding: 24px;
      box-sizing: border-box;
      width: 100%;
  }

  .icon-btn {
      background: none;
      border: none;
      color: var(--text-secondary);
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 6px;
      border-radius: 6px;
      transition: background 0.2s, color 0.2s;
  }

  .icon-btn:hover {
      background-color: var(--bg-panel-hover);
      color: var(--text-primary);
  }

  .avatar {
      width: 32px;
      height: 32px;
      border-radius: 16px;
      background-color: var(--text-primary);
      color: var(--bg-app);
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: bold;
      cursor: pointer;
  }
</style>
