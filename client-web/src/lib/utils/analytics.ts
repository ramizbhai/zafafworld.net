import { env } from '$env/dynamic/public';
import { browser } from '$app/environment';

export async function trackBlogFunnelEvent(eventType: 'afrah_start' | 'inquiry_start' | 'booking_conversion') {
  if (!browser) return;
  const blogSlug = sessionStorage.getItem('zafaf_blog_attribution');
  if (!blogSlug) return; // No attribution

  try {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    await fetch(`${API_BASE}/api/v1/public/blogs/${blogSlug}/track`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ event_type: eventType })
    });
  } catch (err) {
    console.error('[Analytics] Failed to track funnel event:', err);
  }
}
