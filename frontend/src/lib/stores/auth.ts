import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

interface AuthState {
    isAuthenticated: boolean;
    token: string | null;
    user: any | null;
}

const store = writable<AuthState>({
    isAuthenticated: false,
    token: null,
    user: null
});

export const auth = {
    subscribe: store.subscribe,
    get: () => get(store),
    init: () => {
        if (!browser) return;
        const token = localStorage.getItem('bms_token');
        const userStr = localStorage.getItem('bms_user');
        if (token && userStr) {
            try {
                store.set({ isAuthenticated: true, token, user: JSON.parse(userStr) });
            } catch (e) {
                store.set({ isAuthenticated: false, token: null, user: null });
            }
        }
    },
    login: (userId: string, username: string, token: string) => {
        const user = { id: userId, username };
        if (browser) {
            localStorage.setItem('bms_token', token);
            localStorage.setItem('bms_user', JSON.stringify(user));
        }
        store.set({ isAuthenticated: true, token, user });
    },
    logout: () => {
        if (browser) {
            localStorage.removeItem('bms_token');
            localStorage.removeItem('bms_user');
        }
        store.set({ isAuthenticated: false, token: null, user: null });
    }
};
