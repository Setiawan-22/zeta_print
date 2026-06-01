<script lang="ts">
    import { eventManager } from '$lib/stores/events.svelte';
    import { X, Info, AlertTriangle, CheckCircle } from '@lucide/svelte';
    import { fly, fade } from 'svelte/transition';

    function getIcon(type: string) {
        switch (type.toLowerCase()) {
            case 'success': return CheckCircle;
            case 'error': return AlertTriangle;
            case 'warning': return AlertTriangle;
            default: return Info;
        }
    }

    function getColor(type: string) {
        switch (type.toLowerCase()) {
            case 'success': return 'var(--status-green)';
            case 'error': return 'var(--status-red)';
            case 'warning': return 'var(--status-yellow)';
            default: return 'var(--accent-blue)';
        }
    }
</script>

<div class="toast-container">
    {#each eventManager.toasts as toast (toast.id)}
        {@const Icon = getIcon(toast.event_type)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
            class="toast" 
            in:fly={{ y: 50, duration: 300 }} 
            out:fade={{ duration: 200 }}
            onclick={() => eventManager.removeToast(toast.id)}
        >
            <div class="toast-icon" style="color: {getColor(toast.event_type)}">
                <Icon size={20} />
            </div>
            <div class="toast-content">
                <div class="toast-title">{toast.event_type.toUpperCase()}</div>
                <div class="toast-message">{toast.message}</div>
            </div>
            <button class="close-btn" onclick={(e) => { e.stopPropagation(); eventManager.removeToast(toast.id); }}>
                <X size={16} />
            </button>
        </div>
    {/each}
</div>

<style>
    .toast-container {
        position: fixed;
        bottom: 24px;
        right: 24px;
        display: flex;
        flex-direction: column;
        gap: 12px;
        z-index: 1000;
        pointer-events: none;
    }

    .toast {
        background-color: var(--bg-panel);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 16px;
        width: 320px;
        display: flex;
        align-items: flex-start;
        gap: 12px;
        box-shadow: 0 8px 24px rgba(0,0,0,0.15);
        pointer-events: auto;
        cursor: pointer;
        backdrop-filter: blur(10px);
        transition: transform 0.2s, box-shadow 0.2s;
    }

    .toast:hover {
        transform: translateY(-2px);
        box-shadow: 0 12px 32px rgba(0,0,0,0.2);
    }

    .toast-icon {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        padding-top: 2px;
    }

    .toast-content {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .toast-title {
        font-weight: 600;
        font-size: 0.875rem;
        color: var(--text-primary);
        line-height: 1.2;
    }

    .toast-message {
        font-size: 0.875rem;
        color: var(--text-secondary);
        line-height: 1.4;
    }

    .close-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0.5;
        transition: opacity 0.2s, background-color 0.2s;
    }

    .close-btn:hover {
        opacity: 1;
        background-color: var(--bg-panel-hover);
        color: var(--text-primary);
    }
</style>
