/**
 * ZafafWorld Unified API Client
 * 
 * All frontend and SSR data fetching MUST go through this client.
 * It handles base URL resolution, authorization headers, and unified error handling.
 */
import { publicApiBase } from '$lib/utils/env.js';

export interface ApiOptions extends RequestInit {
    /** 
     * The fetch function to use. In SvelteKit +page.server.ts load functions, 
     * always pass the augmented `fetch` provided by SvelteKit to ensure SSR works.
     */
    fetch?: typeof fetch;
    /** The auth session token to attach (optional, handled automatically if passed) */
    token?: string;
    /** Whether the call is happening on the server-side */
    isServer?: boolean;
}

export class ApiError extends Error {
    public status: number;
    public data: any;
    constructor(status: number, data: any, message?: string) {
        super(message || 'API Error');
        this.status = status;
        this.data = data;
        this.name = 'ApiError';
    }
}

export const apiClient = {
    async request<T>(endpoint: string, options: ApiOptions = {}): Promise<T> {
        const { fetch: customFetch = fetch, token, isServer = false, ...init } = options;
        
        const baseUrl = publicApiBase();
        const url = endpoint.startsWith('http') ? endpoint : `${baseUrl}${endpoint.startsWith('/') ? '' : '/'}${endpoint}`;
        
        const headers = new Headers(init.headers);
        if (token && !headers.has('Authorization')) {
            headers.set('Authorization', `Bearer ${token}`);
        }
        if (!headers.has('Content-Type') && !(init.body instanceof FormData)) {
            headers.set('Content-Type', 'application/json');
        }

        const response = await customFetch(url, { ...init, headers });

        let data: any;
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('application/json')) {
            data = await response.json();
        } else {
            data = await response.text();
        }

        if (!response.ok) {
            throw new ApiError(response.status, data, data?.message || response.statusText);
        }

        return data as T;
    },

    async get<T>(endpoint: string, options?: ApiOptions): Promise<T> {
        return this.request<T>(endpoint, { ...options, method: 'GET' });
    },

    async post<T>(endpoint: string, body: any, options?: ApiOptions): Promise<T> {
        return this.request<T>(endpoint, { 
            ...options, 
            method: 'POST', 
            body: body instanceof FormData ? body : JSON.stringify(body) 
        });
    },

    async put<T>(endpoint: string, body: any, options?: ApiOptions): Promise<T> {
        return this.request<T>(endpoint, { 
            ...options, 
            method: 'PUT', 
            body: body instanceof FormData ? body : JSON.stringify(body) 
        });
    },

    async patch<T>(endpoint: string, body: any, options?: ApiOptions): Promise<T> {
        return this.request<T>(endpoint, { 
            ...options, 
            method: 'PATCH', 
            body: body instanceof FormData ? body : JSON.stringify(body) 
        });
    },

    async delete<T>(endpoint: string, options?: ApiOptions): Promise<T> {
        return this.request<T>(endpoint, { ...options, method: 'DELETE' });
    }
};
