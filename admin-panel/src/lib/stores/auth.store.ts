import { writable } from 'svelte/store';

export interface AdminUser {
    id: string;
    email: string;
    role: string;
    scopes?: string[];
    first_name?: string;
    last_name?: string;
}

interface AuthState {
    user: AdminUser | null;
    isAuthenticated: boolean;
    isInitialized: boolean;
}

function createAuthStore() {
    const { subscribe, set, update } = writable<AuthState>({
        user: null,
        isAuthenticated: false,
        isInitialized: false
    });

    return {
        subscribe,
        
        initialize: (user: AdminUser | null) => {
            set({
                user,
                isAuthenticated: !!user,
                isInitialized: true
            });
        },
        
        login: (user: AdminUser) => {
            set({
                user,
                isAuthenticated: true,
                isInitialized: true
            });
        },
        
        logout: () => {
            set({
                user: null,
                isAuthenticated: false,
                isInitialized: true
            });
            // Attempt to clean up session cookie and redirect
            if (typeof window !== 'undefined') {
                document.cookie = 'zafaf_admin_session=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;';
                window.location.href = '/login';
            }
        }
    };
}

export const authStore = createAuthStore();
