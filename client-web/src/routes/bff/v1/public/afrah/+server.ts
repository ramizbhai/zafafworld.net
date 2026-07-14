/**
 * BFF Endpoint for Afrah Concierge Inquiries
 * 
 * Dedicated proxy for Afrah form submissions.
 * Rate limited per-phone (3 req/hr) + per-IP (10 req/hr).
 * Validates and sanitizes before forwarding to backend.
 */

import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';

// ── Rate Limiting ─────────────────────────────────────────────────────────────

interface RateLimitEntry {
  count: number;
  resetAt: number;
}

const rateLimitMap = new Map<string, RateLimitEntry>();
const MAX_PHONE_REQUESTS = 3;
const MAX_IP_REQUESTS = 10;
const WINDOW_MS = 60 * 60 * 1000; // 1 hour

function checkRateLimit(key: string, maxRequests: number): { limited: boolean; retryAfter?: number } {
  const now = Date.now();
  const entry = rateLimitMap.get(key);

  if (!entry || now > entry.resetAt) {
    rateLimitMap.set(key, { count: 1, resetAt: now + WINDOW_MS });
    return { limited: false };
  }

  if (entry.count >= maxRequests) {
    const retryAfter = Math.ceil((entry.resetAt - now) / 1000);
    return { limited: true, retryAfter };
  }

  entry.count++;
  return { limited: false };
}

// Cleanup every 10 minutes
setInterval(() => {
  const now = Date.now();
  for (const [key, entry] of rateLimitMap) {
    if (now > entry.resetAt) rateLimitMap.delete(key);
  }
}, 10 * 60 * 1000);

// ── Validation ────────────────────────────────────────────────────────────────

const PHONE_REGEX = /^\+[1-9]\d{6,14}$/;
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validateAfrahPayload(body: any): { error?: string; sanitized?: any } {
  const stripHtml = (s: string) => s.replace(/<[^>]*>/g, '').trim();

  // Name — required, 3-100 chars
  if (!body.name || typeof body.name !== 'string' || body.name.trim().length < 3) {
    return { error: 'Name is required (min 3 characters)' };
  }
  if (body.name.trim().length > 100) {
    return { error: 'Name must not exceed 100 characters' };
  }

  // Phone — required, international format
  if (!body.phone || typeof body.phone !== 'string') {
    return { error: 'Phone number is required' };
  }
  if (!PHONE_REGEX.test(body.phone.trim())) {
    return { error: 'Invalid phone format. Use international format: +966XXXXXXXXX' };
  }

  // is_whatsapp — boolean
  if (typeof body.isWhatsapp !== 'boolean') {
    return { error: 'WhatsApp preference is required' };
  }

  // event_date — required, must be future
  if (!body.eventDate || typeof body.eventDate !== 'string') {
    return { error: 'Event date is required' };
  }
  const d = new Date(body.eventDate);
  if (isNaN(d.getTime())) {
    return { error: 'Invalid date format' };
  }
  if (d <= new Date()) {
    return { error: 'Event date must be in the future' };
  }

  // Email — optional
  if (body.email && typeof body.email === 'string' && body.email.trim().length > 0) {
    if (!EMAIL_REGEX.test(body.email.trim())) {
      return { error: 'Invalid email format' };
    }
  }

  // Message — optional, max 500
  if (body.message && typeof body.message === 'string' && body.message.trim().length > 500) {
    return { error: 'Message must not exceed 500 characters' };
  }

  const sanitized = {
    name: stripHtml(String(body.name)),
    phone: String(body.phone).trim(),
    isWhatsapp: body.isWhatsapp,
    eventDate: body.eventDate,
    ...(body.email ? { email: String(body.email).trim() } : {}),
    message: body.message ? stripHtml(String(body.message)) : '',
  };

  return { sanitized };
}

// ── Handler ───────────────────────────────────────────────────────────────────

export const POST = async ({ request, getClientAddress }: RequestEvent) => {
  // 1. IP rate limit
  const ip = getClientAddress();
  const ipCheck = checkRateLimit(`afrah:ip:${ip}`, MAX_IP_REQUESTS);
  if (ipCheck.limited) {
    return json(
      { status: 'error', message: 'Too many submissions. Please try again later.', retry_after: ipCheck.retryAfter },
      { status: 429, headers: { 'Retry-After': String(ipCheck.retryAfter) } }
    );
  }

  // 2. Parse body
  let body: any;
  try {
    body = await request.json();
  } catch {
    return json({ status: 'error', message: 'Invalid request body' }, { status: 400 });
  }

  // 3. Validate
  const { error, sanitized } = validateAfrahPayload(body);
  if (error || !sanitized) {
    return json({ status: 'error', message: error || 'Validation failed' }, { status: 400 });
  }

  // 4. Per-phone rate limit
  const phoneCheck = checkRateLimit(`afrah:phone:${sanitized.phone}`, MAX_PHONE_REQUESTS);
  if (phoneCheck.limited) {
    return json(
      { status: 'error', message: 'Too many submissions. Please try again later.', retry_after: phoneCheck.retryAfter },
      { status: 429, headers: { 'Retry-After': String(phoneCheck.retryAfter) } }
    );
  }

  // 5. Forward to backend
  const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
  const endpoint = `${API_BASE}/api/v1/public/afrah/inquiry`;

  try {
    console.log(`[Afrah BFF] Forwarding Afrah inquiry to ${endpoint}`);
    const backendRes = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(sanitized),
      signal: AbortSignal.timeout(10000),
    });

    let backendData: any;
    const contentType = backendRes.headers.get('content-type') || '';
    if (contentType.includes('application/json')) {
      backendData = await backendRes.json().catch(() => null);
    }

    if (!backendData) {
      const text = await backendRes.text().catch(() => '');
      backendData = { status: 'error', message: text || `Backend returned status ${backendRes.status}` };
    }

    return json(backendData, { status: backendRes.status });
  } catch (err: any) {
    console.error('[Afrah BFF] Backend request failed:', err?.message || err);
    return json(
      { status: 'error', message: 'Service temporarily unavailable. Please try again.' },
      { status: 502 }
    );
  }
};
