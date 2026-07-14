/**
 * wizardFetch.ts — Hardened fetch wrapper for the listing wizard steps.
 *
 * Wraps the native `fetch()` with:
 *  1. A 20-second AbortSignal timeout (prevents hanging requests)
 *  2. User-friendly error messages for network / timeout / CORS failures
 *  3. Consistent error contract for the step catch blocks
 *
 * This is intentionally lightweight — it does NOT duplicate the full safeFetch
 * retry/correlation-ID logic because wizard mutations are non-idempotent PUT/PATCH
 * requests that should NOT be blindly retried.
 */

const WIZARD_TIMEOUT_MS = 20_000; // 20 seconds

/**
 * Drop-in replacement for `fetch()` in wizard step submit handlers.
 * Signature is identical to `fetch()` so the migration is a one-word change.
 */
export async function wizardFetch(
  input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> {
  // Merge a timeout signal with any existing signal from the caller
  const timeoutController = new AbortController();
  const timeoutId = setTimeout(() => timeoutController.abort(), WIZARD_TIMEOUT_MS);

  // If the caller already supplied a signal, chain them
  if (init?.signal) {
    init.signal.addEventListener('abort', () => timeoutController.abort());
  }

  try {
    const response = await fetch(input, {
      ...init,
      signal: timeoutController.signal,
    });
    return response;
  } catch (err: any) {
    // Distinguish timeout from network errors for clearer UX
    if (err?.name === 'AbortError') {
      throw new Error(
        'Request timed out. Please check your internet connection and try again.',
      );
    }

    // NetworkError / TypeError from CORS preflight failures or offline
    if (err?.name === 'TypeError' || err?.message?.includes('NetworkError')) {
      throw new Error(
        'Unable to reach the server. Please check your internet connection and try again.',
      );
    }

    // Re-throw anything else as-is
    throw err;
  } finally {
    clearTimeout(timeoutId);
  }
}
