<script lang="ts">
    import '../app.css';
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';

    let { children } = $props();

    onMount(() => {
        auth.init();

        const unsubscribe = auth.subscribe(state => {
            if (!state.isAuthenticated && !$page.url.pathname.startsWith('/login')) {
                goto('/login');
            } else if (state.isAuthenticated && $page.url.pathname === '/login') {
                goto('/dashboard');
            }
        });

        return unsubscribe;
    });
</script>

<div class="app-container">
    {@render children()}
</div>

<style>
    .app-container {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
    }
</style>
