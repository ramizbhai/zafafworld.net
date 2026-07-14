import { eventBus } from '../events/eventBus';

export class ApiError extends Error {
  public status: number;
  public code: string;
  public message: string;
  public metadata?: any;

  constructor(status: number, code: string, message: string, metadata?: any) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.message = message;
    this.metadata = metadata;
  }
}

export interface ApiRequestOptions extends RequestInit {
  timeoutMs?: number;
  maxRetries?: number;
}

class ApiClient {
  private getBaseUrl(): string {
    // Rely on process.env or window depending on environment, or generic default
    if (typeof process !== 'undefined' && process.env.PUBLIC_API_URL) {
      return process.env.PUBLIC_API_URL;
    }
    if (typeof window !== 'undefined' && (window as any).__PUBLIC_API_URL__) {
       return (window as any).__PUBLIC_API_URL__;
    }
    // SvelteKit $env/dynamic/public can't be easily imported outside components without context sometimes,
    // so we'll expect the base to be relative if running on client, or we could pass the full path.
    return ''; 
  }

  private generateRequestId(): string {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
      return `req_${crypto.randomUUID()}`;
    }
    return `req_${Math.random().toString(36).substring(2, 11)}_${Date.now()}`;
  }

  public async request<T>(endpoint: string, options: ApiRequestOptions = {}): Promise<T> {
    const { timeoutMs = 15000, maxRetries = 2, ...initOptions } = options;
    const method = (initOptions.method || 'GET').toUpperCase();
    const isSafeMethod = method === 'GET' || method === 'HEAD';

    const baseUrl = this.getBaseUrl();
    const url = endpoint.startsWith('http') ? endpoint : `${baseUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;

    const headers = new Headers(initOptions.headers || {});
    if (!headers.has('X-Request-ID')) {
      headers.set('X-Request-ID', this.generateRequestId());
    }

    let retriesLeft = isSafeMethod ? maxRetries : 0;
    let lastError: any = null;

    while (retriesLeft >= 0) {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

      if (initOptions.signal) {
        initOptions.signal.addEventListener('abort', () => controller.abort());
      }

      try {
        const response = await fetch(url, {
          ...initOptions,
          headers,
          signal: controller.signal,
        });
        clearTimeout(timeoutId);

        const contentType = response.headers.get('content-type') || '';
        const isJson = contentType.includes('application/json');

        if (!response.ok) {
          let errorCode = 'SERVER_ERROR';
          let errorMessage = 'An operation failed on the server.';
          let metadata = undefined;

          if (isJson) {
            const body = await response.json();

            if (response.status === 402 && typeof window !== 'undefined') {
              eventBus.emit('402_PAYMENT_REQUIRED', {
                limitType: body.errors?.limit_type || 'unknown',
                currentTier: body.errors?.current_tier || 'Free',
                message: body.message || 'Subscription limit reached'
              });
            }

            errorCode = body.error_type || body.code || errorCode;
            errorMessage = body.message || errorMessage;
            metadata = body.errors || body.metadata;
          } else {
            errorMessage = await response.text();
            errorCode = 'NON_JSON_ERROR';
          }

          const apiError = new ApiError(response.status, errorCode, errorMessage, metadata);

          if (isSafeMethod && response.status >= 500 && retriesLeft > 0) {
            lastError = apiError;
            retriesLeft--;
            await new Promise((res) => setTimeout(res, 300 * (maxRetries - retriesLeft)));
            continue;
          }
          
          throw apiError;
        }

        if (isJson) {
          const body = await response.json();
          // We assume a generic response wrapper if 'data' and 'status' exist
          if (body && typeof body === 'object' && 'data' in body && 'success' in body) {
            return body.data as T;
          }
          return body as T;
        }

        return (await response.text()) as unknown as T;
      } catch (err: any) {
        clearTimeout(timeoutId);

        if (err instanceof ApiError) {
          throw err;
        }

        const isAbort = err.name === 'AbortError';
        const errorStatus = isAbort ? 408 : 500;
        const errorCode = isAbort ? 'TIMEOUT_ERROR' : 'CONNECTION_FAILED';
        const errorMessage = isAbort
          ? `Request timed out after ${timeoutMs}ms.`
          : 'Unable to communicate with the remote server. Please check your network connection.';

        const networkError = new ApiError(errorStatus, errorCode, errorMessage);

        if (isSafeMethod && retriesLeft > 0) {
          lastError = networkError;
          retriesLeft--;
          await new Promise((res) => setTimeout(res, 300 * (maxRetries - retriesLeft)));
          continue;
        }

        throw networkError;
      }
    }

    throw lastError || new ApiError(500, 'MAX_RETRIES_EXCEEDED', 'Request failed after maximum retry attempts.');
  }

  // Convenience methods
  public get<T>(endpoint: string, options?: Omit<ApiRequestOptions, 'method'>): Promise<T> {
    return this.request<T>(endpoint, { ...options, method: 'GET' });
  }

  public post<T>(endpoint: string, body?: any, options?: Omit<ApiRequestOptions, 'method' | 'body'>): Promise<T> {
    const headers = new Headers(options?.headers);
    if (body && !headers.has('Content-Type') && !(body instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }
    return this.request<T>(endpoint, {
      ...options,
      method: 'POST',
      headers,
      body: body instanceof FormData ? body : JSON.stringify(body),
    });
  }

  public put<T>(endpoint: string, body?: any, options?: Omit<ApiRequestOptions, 'method' | 'body'>): Promise<T> {
    const headers = new Headers(options?.headers);
    if (body && !headers.has('Content-Type') && !(body instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }
    return this.request<T>(endpoint, {
      ...options,
      method: 'PUT',
      headers,
      body: body instanceof FormData ? body : JSON.stringify(body),
    });
  }

  public delete<T>(endpoint: string, options?: Omit<ApiRequestOptions, 'method'>): Promise<T> {
    return this.request<T>(endpoint, { ...options, method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();
