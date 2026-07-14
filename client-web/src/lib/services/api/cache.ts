type CacheEntry = {
    value: any;
    expiry: number;
};

/** Maximum number of entries before the oldest is evicted. Protects against unbounded growth. */
const MAX_CACHE_SIZE = 500;

const store = new Map<string, CacheEntry>();

/** Evict all expired entries. Called periodically and before writes when at capacity. */
function evictExpired(): void {
    const now = Date.now();
    for (const [key, entry] of store) {
        if (entry.expiry <= now) store.delete(key);
    }
}

/** Evict the single oldest (first-inserted) entry. Fallback if all entries are still fresh. */
function evictOldest(): void {
    const firstKey = store.keys().next().value;
    if (firstKey !== undefined) store.delete(firstKey);
}

// Cleanup expired entries every 2 minutes to keep memory usage low between traffic spikes.
setInterval(evictExpired, 2 * 60 * 1000);

/**
 * Retrieve a cached value or execute the fetcher callback to cache and return the fresh value.
 *
 * - TTL: configurable, default 5 minutes
 * - Max size: 500 entries (oldest-first eviction when full)
 */
export async function getCached<T>(
    key: string,
    fetcher: () => Promise<T>,
    ttlMs = 5 * 60 * 1000 // default to 5 minutes
): Promise<T> {
    const now = Date.now();
    const cached = store.get(key);

    if (cached && cached.expiry > now) {
        // Re-insert to mark as recently used (insertion-order LRU approximation)
        store.delete(key);
        store.set(key, cached);
        return cached.value;
    }

    const value = await fetcher();

    // Enforce size limit before inserting
    if (store.size >= MAX_CACHE_SIZE) {
        evictExpired();
        if (store.size >= MAX_CACHE_SIZE) evictOldest();
    }

    store.set(key, { value, expiry: Date.now() + ttlMs });
    return value;
}

/** Manually invalidate a cache entry (e.g. after a write operation). */
export function invalidateCache(key: string): void {
    store.delete(key);
}

/** Return the number of live (non-expired) cache entries. Useful for diagnostics. */
export function cacheSize(): number {
    const now = Date.now();
    let count = 0;
    for (const entry of store.values()) {
        if (entry.expiry > now) count++;
    }
    return count;
}
