/**
 * BFF Rate-Limiting Proxy for Public Inquiries
 * 
 * Security layers:
 * 1. SvelteKit built-in CSRF (Origin header check on same-origin POST)
 * 2. Two-tier server-side rate limiting:
 *    a. Per-IP global: 20 req/IP/hour (anti-abuse / DDoS protection)
 *    b. Per-user-per-vendor: 5 req/phone+listing/hour (business rule)
 * 3. Payload validation & sanitization before forwarding to backend
 * 4. Honeypot field detection for bot spam
 */

import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';
import { apiClient } from '$lib/services/api/client.js';

// ── Rate Limiting ─────────────────────────────────────────────────────────────
// 
// Two-tier rate limiting:
// 1. Per-user-per-vendor: phone+listingId → max 5 inquiries/hour (business rule)
// 2. Per-IP global: IP → max 20 inquiries/hour (anti-abuse / DDoS protection)

interface RateLimitEntry {
  count: number;
  resetAt: number;
}

const rateLimitMap = new Map<string, RateLimitEntry>();

// Per-user-per-vendor limit (the business rule: 5 per user per vendor per hour)
const MAX_USER_VENDOR_REQUESTS = 5;
// Per-IP global limit (anti-abuse: high ceiling so legitimate users aren't blocked)
const MAX_IP_REQUESTS = 20;
const WINDOW_MS = 60 * 60 * 1000; // 1 hour
/** Max entries before oldest key is evicted — prevents OOM during DDoS. */
const MAX_RATELIMIT_ENTRIES = 50_000;

function checkRateLimit(key: string, maxRequests: number): { limited: boolean; retryAfter?: number } {
  const now = Date.now();
  const entry = rateLimitMap.get(key);

  // First request or window expired — start a fresh window
  if (!entry || now > entry.resetAt) {
    // Enforce size limit: evict oldest entry if at capacity
    if (rateLimitMap.size >= MAX_RATELIMIT_ENTRIES) {
      const oldestKey = rateLimitMap.keys().next().value;
      if (oldestKey !== undefined) rateLimitMap.delete(oldestKey);
    }
    rateLimitMap.set(key, { count: 1, resetAt: now + WINDOW_MS });
    return { limited: false };
  }

  // Already at or over the limit — reject
  if (entry.count >= maxRequests) {
    const retryAfter = Math.ceil((entry.resetAt - now) / 1000);
    return { limited: true, retryAfter };
  }

  // Under the limit — allow and increment
  entry.count++;
  return { limited: false };
}

// Periodic cleanup of expired entries (every 10 minutes)
setInterval(() => {
  const now = Date.now();
  for (const [key, entry] of rateLimitMap) {
    if (now > entry.resetAt) {
      rateLimitMap.delete(key);
    }
  }
}, 10 * 60 * 1000);

// ── Validation ────────────────────────────────────────────────────────────────

const UUID_REGEX = /^(?:draft-)?[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const PHONE_REGEX = /^\+[1-9]\d{6,14}$/;
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

interface ValidationErrors {
  [key: string]: string;
}

function validateInquiryPayload(body: any): { errors: ValidationErrors; sanitized: any } {
  const errors: ValidationErrors = {};

  // Honeypot check — if this hidden field has a value, it's a bot
  if (body.website && String(body.website).trim().length > 0) {
    // Silently accept but don't process — don't reveal honeypot to bots
    return { errors: { __honeypot: 'bot_detected' }, sanitized: null };
  }

  // Draft intercept - mock success for draft listing previews
  if (body.listing_id && typeof body.listing_id === 'string' && body.listing_id.startsWith('draft-')) {
    return { errors: { __draft: 'draft_preview' }, sanitized: null };
  }

  // listing_id — required for listing inquiries, optional for general/concierge
  const inquiryType = body.type || 'listing';
  if (inquiryType === 'listing') {
    if (!body.listing_id && !body.vendorId) {
      errors.listing_id = 'Listing ID is required';
    } else if (body.listing_id && !UUID_REGEX.test(body.listing_id)) {
      errors.listing_id = 'Invalid listing ID format';
    }
  }

  // name — required, 3-100 chars
  if (!body.name || typeof body.name !== 'string') {
    errors.name = 'Name is required';
  } else if (body.name.trim().length < 3) {
    errors.name = 'Name must be at least 3 characters';
  } else if (body.name.trim().length > 100) {
    errors.name = 'Name must not exceed 100 characters';
  }

  // mobile — required, international format
  if (!body.mobile || typeof body.mobile !== 'string') {
    errors.mobile = 'Phone number is required';
  } else if (!PHONE_REGEX.test(body.mobile.trim())) {
    errors.mobile = 'Invalid phone format. Use international format: +966XXXXXXXXX';
  }

  // is_whatsapp — required, boolean
  if (typeof body.is_whatsapp !== 'boolean') {
    errors.is_whatsapp = 'WhatsApp preference is required';
  }

  // email — optional, but if provided must be valid
  if (body.email && typeof body.email === 'string' && body.email.trim().length > 0) {
    if (!EMAIL_REGEX.test(body.email.trim())) {
      errors.email = 'Invalid email format';
    }
  }

  // message — optional, max 500 chars
  if (body.message && typeof body.message === 'string' && body.message.trim().length > 500) {
    errors.message = 'Message must not exceed 500 characters';
  }

  // event_date — optional, must be future date
  if (body.event_date && typeof body.event_date === 'string') {
    const d = new Date(body.event_date);
    if (isNaN(d.getTime())) {
      errors.event_date = 'Invalid date format';
    } else if (d <= new Date()) {
      errors.event_date = 'Event date must be in the future';
    }
  }

  // guest_count — optional, positive integer
  if (body.guest_count !== undefined && body.guest_count !== null && body.guest_count !== '') {
    const gc = Number(body.guest_count);
    if (!Number.isInteger(gc) || gc <= 0) {
      errors.guest_count = 'Guest count must be a positive number';
    }
  }

  // Build sanitized payload — strip HTML from all string fields
  const stripHtml = (s: string) => s.replace(/<[^>]*>/g, '').trim();

  let eventDateVal = body.event_date || body.eventDate || '';
  if (!eventDateVal) {
    const nextYear = new Date();
    nextYear.setFullYear(nextYear.getFullYear() + 1);
    eventDateVal = nextYear.toISOString().split('T')[0];
  }

  const sanitized = {
    ...(body.listing_id || body.listingId ? { listingId: body.listing_id || body.listingId } : {}),
    ...(body.vendor_id || body.vendorId ? { vendorId: body.vendor_id || body.vendorId } : {}),
    ...(body.name ? { name: stripHtml(String(body.name)) } : {}),
    phone: String(body.mobile || body.phone || '').trim(),
    ...(body.email ? { email: String(body.email).trim() } : {}),
    message: body.message ? stripHtml(String(body.message)) : '',
    eventDate: eventDateVal,
    guestCount: body.guest_count || body.guestCount ? Math.floor(Number(body.guest_count || body.guestCount)) : 1,
  };

  return { errors, sanitized };
}

// ── Handler ───────────────────────────────────────────────────────────────────

export const POST = async ({ request, getClientAddress }: RequestEvent) => {
  // 1a. Global IP rate limit — anti-abuse layer (high ceiling)
  const ip = getClientAddress();
  const ipCheck = checkRateLimit(`ip:${ip}`, MAX_IP_REQUESTS);
  if (ipCheck.limited) {
    return json(
      {
        status: 'error',
        message: 'Too many submissions. Please try again later.',
        retry_after: ipCheck.retryAfter,
      },
      {
        status: 429,
        headers: { 'Retry-After': String(ipCheck.retryAfter) },
      }
    );
  }

  // 2. Parse request body
  let body: any;
  try {
    body = await request.json();
  } catch {
    return json(
      { status: 'error', message: 'Invalid request body' },
      { status: 400 }
    );
  }

  // 3. Validate & sanitize
  const { errors, sanitized } = validateInquiryPayload(body);

  // Honeypot triggered — return fake success to fool bots
  if (errors.__honeypot) {
    return json(
      { status: 'success', tracking_id: 'INQ-0000-FAKE', message: 'Inquiry submitted.' },
      { status: 201 }
    );
  }

  // Draft triggered - return fake success for draft previews
  if (errors.__draft) {
    return json(
      { status: 'success', tracking_id: 'INQ-DRAFT-TEST', message: 'Test inquiry submitted (Draft mode).' },
      { status: 201 }
    );
  }

  if (Object.keys(errors).length > 0) {
    return json(
      { status: 'error', message: 'Validation failed', errors },
      { status: 400 }
    );
  }

  // 1b. Per-user-per-vendor rate limit — business rule (5 inquiries/user/vendor/hour)
  //     Uses phone + listingId as composite key to identify unique user-vendor pairs
  const userVendorKey = `uv:${sanitized.phone || ''}:${sanitized.listingId || sanitized.vendorId || 'general'}`;
  const userVendorCheck = checkRateLimit(userVendorKey, MAX_USER_VENDOR_REQUESTS);
  if (userVendorCheck.limited) {
    return json(
      {
        status: 'error',
        message: 'Too many submissions. Please try again later.',
        retry_after: userVendorCheck.retryAfter,
      },
      {
        status: 429,
        headers: { 'Retry-After': String(userVendorCheck.retryAfter) },
      }
    );
  }

  // 4. Forward to backend — determine authenticated vs guest route
  const authHeader = request.headers.get('authorization');
  const token = authHeader ? authHeader.replace('Bearer ', '').trim() : undefined;
  
  const endpoint = authHeader 
    ? '/api/v1/public/inquiries' 
    : '/api/v1/public/inquiries/guest';

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (authHeader) {
    headers['Authorization'] = authHeader;
  }

  try {
    console.log(`[Inquiry BFF] Forwarding inquiry to ${endpoint} (Auth: ${!!authHeader})`);
    
    // apiClient handles errors by throwing ApiError, which we catch below
    const backendData = await apiClient.post<any>(endpoint, sanitized, {
      token,
      isServer: true,
      signal: AbortSignal.timeout(10000)
    });

    return json(backendData, { status: 201 });
  } catch (err: any) {
    console.error('[Inquiry BFF] Backend request failed:', err?.message || err);
    if (err.name === 'ApiError') {
      return json(err.data, { status: err.status });
    }
    return json(
      { status: 'error', message: 'Service temporarily unavailable. Please try again.' },
      { status: 502 }
    );
  }
};
