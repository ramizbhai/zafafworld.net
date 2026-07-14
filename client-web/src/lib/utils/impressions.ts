/**
 * impressions.ts — Centralized Impression Tracking Service
 *
 * Features:
 * - IntersectionObserver with 50% visibility threshold
 * - 1-second minimum visibility requirement (no scroll-through false positives)
 * - Per-page deduplication (same listing tracked once per session)
 * - Batched flush every 5 seconds (minimizes network overhead)
 * - navigator.sendBeacon on page unload (no data loss)
 * - Fire-and-forget: never blocks UI, errors silently logged
 */

import { browser } from '$app/environment';

// ── Types ─────────────────────────────────────────────────────────────────────

interface ImpressionEvent {
  listing_id: string;
  tier_id: string;
  timestamp: number;
}

interface ImpressionActionParams {
  listingId: string;
  tierId: string;
}

// ── Singleton Tracker ─────────────────────────────────────────────────────────

class ImpressionTracker {
  private seen = new Set<string>();
  private queue: ImpressionEvent[] = [];
  private observer: IntersectionObserver | null = null;
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private timers = new Map<Element, ReturnType<typeof setTimeout>>();

  constructor() {
    if (!browser) return;

    // IntersectionObserver: fire when 50% of card is visible
    this.observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const el = entry.target as HTMLElement;
          const listingId = el.dataset.impressionId;
          const tierId = el.dataset.impressionTier;

          if (entry.isIntersecting) {
            // Start 1-second timer — only track if visible for 1s+
            if (!this.timers.has(el) && listingId && !this.seen.has(listingId)) {
              const timer = setTimeout(() => {
                this.track(listingId, tierId || 'free');
                this.timers.delete(el);
              }, 1000);
              this.timers.set(el, timer);
            }
          } else {
            // Scrolled away before 1s — cancel
            const timer = this.timers.get(el);
            if (timer) {
              clearTimeout(timer);
              this.timers.delete(el);
            }
          }
        }
      },
      { threshold: 0.5 }
    );

    // Flush batch every 5 seconds
    this.flushTimer = setInterval(() => this.flush(), 5000);

    // Flush on page unload via sendBeacon (no data loss)
    if (typeof window !== 'undefined') {
      window.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'hidden') {
          this.flush();
        }
      });

      window.addEventListener('beforeunload', () => {
        this.flush();
      });
    }
  }

  /** Register an element for observation */
  observe(el: HTMLElement, listingId: string, tierId: string): void {
    if (!this.observer) return;
    el.dataset.impressionId = listingId;
    el.dataset.impressionTier = tierId;
    this.observer.observe(el);
  }

  /** Unregister an element */
  unobserve(el: HTMLElement): void {
    if (!this.observer) return;
    this.observer.unobserve(el);
    const timer = this.timers.get(el);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(el);
    }
  }

  /** Add a verified impression to the batch queue */
  private track(listingId: string, tierId: string): void {
    if (this.seen.has(listingId)) return;
    this.seen.add(listingId);
    this.queue.push({
      listing_id: listingId,
      tier_id: tierId,
      timestamp: Date.now(),
    });
  }

  /** Flush the batch to the server */
  private flush(): void {
    if (this.queue.length === 0) return;

    const batch = [...this.queue];
    this.queue = [];

    // Use sendBeacon for reliability (works even during page unload)
    if (typeof navigator !== 'undefined' && navigator.sendBeacon) {
      const sent = navigator.sendBeacon(
        '/api/v1/analytics/impressions',
        JSON.stringify({ impressions: batch })
      );
      if (!sent) {
        // Fallback to fetch if sendBeacon fails
        this.fetchFlush(batch);
      }
    } else {
      this.fetchFlush(batch);
    }
  }

  /** Fallback flush via fetch (fire-and-forget) */
  private fetchFlush(batch: ImpressionEvent[]): void {
    // Fire and forget to SvelteKit BFF (handles buffering & retry logic)
    fetch('/bff/v1/analytics/impressions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ impressions: batch }),
      keepalive: true,
    }).catch((err) => {
      console.error('[Impressions] Flush failed:', err);
    });
  }

  /** Cleanup — call when tracker is no longer needed */
  destroy(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    for (const timer of this.timers.values()) {
      clearTimeout(timer);
    }
    this.timers.clear();
    this.flush(); // Final flush
  }
}

// ── Singleton Instance ────────────────────────────────────────────────────────

let tracker: ImpressionTracker | null = null;

function getTracker(): ImpressionTracker {
  if (!tracker) {
    tracker = new ImpressionTracker();
  }
  return tracker;
}

// ── Svelte Action ─────────────────────────────────────────────────────────────

/**
 * Svelte action for impression tracking.
 * Usage: <article use:impressionAction={{ listingId: 'xxx', tierId: 'diamond' }}>
 */
export function impressionAction(
  node: HTMLElement,
  params: ImpressionActionParams
) {
  if (!browser) return;

  const t = getTracker();
  t.observe(node, params.listingId, params.tierId);

  return {
    update(newParams: ImpressionActionParams) {
      t.unobserve(node);
      t.observe(node, newParams.listingId, newParams.tierId);
    },
    destroy() {
      t.unobserve(node);
    },
  };
}
