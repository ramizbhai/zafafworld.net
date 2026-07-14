/**
 * Impressions BFF Proxy
 *
 * Receives batched impression events from the frontend and forwards
 * them to the backend analytics endpoint. Fire-and-forget — returns
 * 204 immediately regardless of backend response.
 */

import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';

export const POST = async ({ request }: RequestEvent) => {
  let body: any;
  try {
    body = await request.json();
  } catch {
    return new Response(null, { status: 400 });
  }

  // Validate: must have an impressions array
  if (!body.impressions || !Array.isArray(body.impressions) || body.impressions.length === 0) {
    return new Response(null, { status: 400 });
  }

  // Validate each impression has required fields
  const validImpressions = body.impressions.filter(
    (imp: any) =>
      imp.listing_id &&
      typeof imp.listing_id === 'string' &&
      imp.tier_id &&
      typeof imp.tier_id === 'string' &&
      imp.timestamp &&
      typeof imp.timestamp === 'number'
  );

  if (validImpressions.length === 0) {
    return new Response(null, { status: 400 });
  }

  // Forward to backend — fire and forget, don't block response
  const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

  // Non-blocking forward — we don't await the result
  fetch(`${API_BASE}/api/v1/analytics/impressions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ impressions: validImpressions }),
  }).catch((err) => {
    // Backend may not have this endpoint yet — silently log
    console.warn('[Impressions BFF] Backend forward failed (endpoint may not exist yet):', err.message);
  });

  // Return 204 immediately — client doesn't need to wait for backend
  return new Response(null, { status: 204 });
};
