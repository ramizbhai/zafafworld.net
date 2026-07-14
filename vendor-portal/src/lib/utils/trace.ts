/**
 * Wizard-scoped distributed trace ID utility.
 *
 * A single trace ID is created when the wizard initializes and stored in
 * sessionStorage. All wizard step API calls inject this ID as X-Trace-ID
 * so the entire wizard flow (Step 1 → Step N → Submit) can be correlated
 * in server logs using one identifier.
 *
 * The trace ID is cleared when the wizard resets (listing submitted or abandoned).
 */

const TRACE_KEY = 'wizard_trace_id';

/** Returns the existing wizard trace ID, or creates a new one and stores it. */
export function getOrCreateTraceId(): string {
  if (typeof sessionStorage === 'undefined') {
    return generateId();
  }
  let id = sessionStorage.getItem(TRACE_KEY);
  if (!id) {
    id = generateId();
    sessionStorage.setItem(TRACE_KEY, id);
  }
  return id;
}

/** Clears the wizard trace ID from session storage (call on wizard reset/submit). */
export function clearTraceId(): void {
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.removeItem(TRACE_KEY);
  }
}

/** Returns the current trace ID without creating a new one, or null if not set. */
export function getTraceId(): string | null {
  if (typeof sessionStorage === 'undefined') return null;
  return sessionStorage.getItem(TRACE_KEY);
}

function generateId(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return `wiz_${crypto.randomUUID()}`;
  }
  return `wiz_${Math.random().toString(36).substring(2, 11)}_${Date.now()}`;
}
