import { env } from '$env/dynamic/public';
import { triggerUpgrade } from '../stores/upgradeStore';

export function getApiBaseUrl(): string {
  return env.PUBLIC_API_URL || 'http://localhost:8080';
}

export function getApiUrl(path: string): string {
  const base = getApiBaseUrl();
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${base}${cleanPath}`;
}

export function getWsUrl(): string {
  return env.PUBLIC_WS_URL || 'ws://localhost:8080';
}

export interface ApiResponse<T> {
  success: boolean;
  status: number;
  data: T | null;
  error: {
    type: string;
    message: string;
    metadata?: any;
  } | null;
  requestId?: string;
}

export interface FetchOptions extends RequestInit {
  timeoutMs?: number;
  maxRetries?: number;
}

function generateRequestId(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return `req_${crypto.randomUUID()}`;
  }
  return `req_${Math.random().toString(36).substring(2, 11)}_${Date.now()}`;
}

export async function safeFetch<T>(
  fetchFn: typeof fetch,
  url: string,
  options: FetchOptions = {}
): Promise<ApiResponse<T>> {
  const { timeoutMs = 15000, maxRetries = 2, ...initOptions } = options;
  const method = (initOptions.method || 'GET').toUpperCase();
  const isSafeMethod = method === 'GET' || method === 'HEAD';

  // Correlation ID handling
  const headers = new Headers(initOptions.headers || {});
  if (!headers.has('X-Request-ID')) {
    headers.set('X-Request-ID', generateRequestId());
  }
  const requestId = headers.get('X-Request-ID') || undefined;

  let retriesLeft = isSafeMethod ? maxRetries : 0;
  let lastErrorResult: ApiResponse<T> | null = null;

  while (retriesLeft >= 0) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    // If signal was provided in options, link it
    if (initOptions.signal) {
      initOptions.signal.addEventListener('abort', () => controller.abort());
    }

    try {
      const response = await fetchFn(url, {
        ...initOptions,
        headers,
        signal: controller.signal,
      });
      clearTimeout(timeoutId);

      const contentType = response.headers.get('content-type') || '';

      if (contentType.includes('application/json')) {
        const body = await response.json();

        if (!response.ok) {
          // Global Interception for 402 Payment Required
          if (response.status === 402 && typeof window !== 'undefined') {
            triggerUpgrade(
              body.errors?.limit_type || 'unknown',
              body.errors?.current_tier || 'Free',
              body.message || 'Subscription limit reached'
            );
          }

          const result: ApiResponse<T> = {
            success: false,
            status: response.status,
            data: null,
            error: {
              type: body.error_type || 'SERVER_ERROR',
              message: body.message || 'A database or business logic operation failed.',
              metadata: body.errors,
            },
            requestId,
          };

          // Retry on server-side 5xx errors for safe GET methods
          if (isSafeMethod && response.status >= 500 && retriesLeft > 0) {
            lastErrorResult = result;
            retriesLeft--;
            await new Promise((res) => setTimeout(res, 300 * (maxRetries - retriesLeft)));
            continue;
          }

          return result;
        }

        return {
          success: true,
          status: response.status,
          data: body as T,
          error: null,
          requestId,
        };
      }

      const textBody = await response.text();
      return {
        success: false,
        status: response.status,
        data: null,
        error: {
          type: response.ok ? 'UNEXPECTED_NON_JSON_PAYLOAD' : 'GATEWAY_ERROR',
          message: response.ok
            ? 'The server returned an invalid success response.'
            : `Network error (${response.status}): The gateway returned a non-JSON document.`,
        },
        requestId,
      };
    } catch (err: any) {
      clearTimeout(timeoutId);
      const isAbort = err.name === 'AbortError';
      const result: ApiResponse<T> = {
        success: false,
        status: isAbort ? 408 : 500,
        data: null,
        error: {
          type: isAbort ? 'TIMEOUT_ERROR' : 'CONNECTION_FAILED',
          message: isAbort
            ? `Request timed out after ${timeoutMs}ms.`
            : 'Unable to communicate with the remote server. Please check your network connection.',
        },
        requestId,
      };

      if (isSafeMethod && retriesLeft > 0) {
        lastErrorResult = result;
        retriesLeft--;
        await new Promise((res) => setTimeout(res, 300 * (maxRetries - retriesLeft)));
        continue;
      }

      return result;
    }
  }

  return lastErrorResult || {
    success: false,
    status: 500,
    data: null,
    error: {
      type: 'MAX_RETRIES_EXCEEDED',
      message: 'Request failed after maximum retry attempts.',
    },
    requestId,
  };
}
