<script lang="ts">
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let username = $state("");
  let password = $state("");
  let errorMsg = $state("");
  let loading = $state(false);

  onMount(() => {
      if (auth.get()?.isAuthenticated) {
          goto('/dashboard');
      }
  });

  async function handleLogin(e: Event) {
      e.preventDefault();
      loading = true;
      errorMsg = '';

      try {
          // Dummy login for prototype
          if (username === 'admin' && password === 'admin') {
              auth.login('admin-user', 'admin', 'admin-token-123');
              goto('/dashboard');
          } else {
              errorMsg = 'Invalid username or password';
          }
      } catch (err) {
          errorMsg = 'Server connection failed';
      } finally {
          loading = false;
      }
  }
</script>

<div class="login-container">
  <div class="login-panel panel">
      <div class="logo">
          <span class="logo-icon text-blue">🖨️</span>
          <span class="logo-text text-xl font-bold">ZetaPrint</span>
      </div>
      
      <p class="text-secondary text-sm mb-4">Sign in to manage print servers and ACL</p>

      <form onsubmit={handleLogin} class="login-form">
          <div class="form-group">
              <label for="username" class="text-sm font-medium text-secondary">Username</label>
              <input 
                  type="text" 
                  id="username" 
                  bind:value={username} 
                  required 
                  class="input-dark font-mono"
                  placeholder="admin"
              />
          </div>

          <div class="form-group">
              <label for="password" class="text-sm font-medium text-secondary">Password</label>
              <input 
                  type="password" 
                  id="password" 
                  bind:value={password} 
                  required 
                  class="input-dark font-mono"
                  placeholder="••••••••"
              />
          </div>

          {#if errorMsg}
              <div class="error-msg text-red text-sm">{errorMsg}</div>
          {/if}

          <button type="submit" class="btn-login" disabled={loading}>
              {loading ? 'Authenticating...' : 'Sign In'}
          </button>
      </form>
  </div>
</div>

<style>
  .login-container {
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      background-color: var(--bg-app);
      background-image: 
        radial-gradient(circle at 15% 50%, rgba(88, 166, 255, 0.05), transparent 25%),
        radial-gradient(circle at 85% 30%, rgba(88, 166, 255, 0.05), transparent 25%);
  }

  .login-panel {
      width: 100%;
      max-width: 400px;
      padding: 32px;
      display: flex;
      flex-direction: column;
      align-items: center;
      box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }

  .logo {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 8px;
  }

  .mb-4 { margin-bottom: 24px; }

  .login-form {
      width: 100%;
      display: flex;
      flex-direction: column;
      gap: 16px;
  }

  .form-group {
      display: flex;
      flex-direction: column;
      gap: 8px;
  }

  .input-dark {
      background-color: var(--bg-app);
      border: 1px solid var(--border-color);
      color: var(--text-primary);
      padding: 12px;
      border-radius: 6px;
      outline: none;
      transition: border-color 0.2s;
  }

  .input-dark:focus {
      border-color: var(--accent-blue);
  }

  .btn-login {
      margin-top: 8px;
      background-color: var(--accent-blue);
      color: #000;
      border: none;
      padding: 12px;
      border-radius: 6px;
      font-weight: 600;
      cursor: pointer;
      transition: opacity 0.2s;
  }

  .btn-login:hover {
      opacity: 0.9;
  }

  .btn-login:disabled {
      opacity: 0.5;
      cursor: not-allowed;
  }

  .error-msg {
      background-color: var(--status-red-bg);
      padding: 10px;
      border-radius: 6px;
      text-align: center;
      border: 1px solid rgba(248, 81, 73, 0.3);
  }
</style>
