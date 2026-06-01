import { browser } from '$app/environment';

export interface AppEvent {
    id: string;
    event_type: string;
    message: string;
    timestamp: string;
}

class EventManager {
    history = $state<AppEvent[]>([]);
    toasts = $state<AppEvent[]>([]);
    private eventSource: EventSource | null = null;

    connect() {
        if (!browser || this.eventSource) return;

        console.log('Connecting to SSE...');
        this.eventSource = new EventSource('/api/v1/dashboard/events');

        this.eventSource.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                
                // Ignore background pings for toasts
                if (data.event_type === 'ping') return;

                const newEvent: AppEvent = {
                    id: crypto.randomUUID(),
                    event_type: data.event_type || 'info',
                    message: data.message || '',
                    timestamp: data.timestamp || new Date().toISOString()
                };

                this.addEvent(newEvent);
            } catch (err) {
                console.error('Failed to parse SSE event:', err);
            }
        };

        this.eventSource.onerror = (err) => {
            console.error('SSE connection error. Reconnecting automatically...', err);
        };
    }

    disconnect() {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }

    addEvent(event: AppEvent) {
        this.history = [event, ...this.history].slice(0, 100);
        this.toasts = [...this.toasts, event];

        // Auto remove toast after 5 seconds
        setTimeout(() => {
            this.removeToast(event.id);
        }, 5000);
    }

    removeToast(id: string) {
        this.toasts = this.toasts.filter(t => t.id !== id);
    }
}

export const eventManager = new EventManager();
