export interface ApiResponse<T> {
  success: boolean;
  status: number;
  data: T | null;
  error: {
    type: string;
    message: string;
  } | null;
}

export async function safeFetch<T>(
  fetchFn: typeof fetch,
  url: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  try {
    const response = await fetchFn(url, options);
    const contentType = response.headers.get('content-type') || '';

    // Guard: Verify JSON payload signature before extracting
    if (contentType.includes('application/json')) {
      const body = await response.json();

      if (!response.ok) {
        // Map backend's standardized ErrorResponse DTO
        return {
          success: false,
          status: response.status,
          data: null,
          error: {
            type: body.error_type || 'SERVER_ERROR',
            message: body.message || 'A database or business logic operation failed.'
          }
        };
      }

      return {
        success: true,
        status: response.status,
        data: body as T,
        error: null
      };
    }

    // Fallback: Handle non-JSON payloads (Gateway Timeout, Proxy Stacktraces, or Empty Bodies)
    const textBody = await response.text();
    return {
      success: false,
      status: response.status,
      data: null,
      error: {
        type: response.ok ? 'UNEXPECTED_NON_JSON_PAYLOAD' : 'GATEWAY_ERROR',
        message: response.ok
          ? 'The server returned an invalid success response.'
          : `Network error (${response.status}): The gateway returned a non-JSON document.`
      }
    };
  } catch (err: any) {
    // Guard: Handle physical transport disconnects, DNS failure, or aborted sockets
    return {
      success: false,
      status: 500,
      data: null,
      error: {
        type: 'CONNECTION_FAILED',
        message: 'Unable to communicate with the remote server. Please check your network connection.'
      }
    };
  }
}
