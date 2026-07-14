/**
 * env.ts — Shared environment constant helpers
 *
 * Centralizes PUBLIC_API_URL resolution so it isn't copy-pasted in 40+ files.
 * All server-side code should import `serverApiBase` for internal routing.
 * All client-side code should import `publicApiBase`.
 *
 * Note: Both values resolve identically at runtime; the distinction documents
 * intent and makes future CDN/proxy rewriting trivial.
 */

import { env } from '$env/dynamic/public';

/** Base URL for API calls — works in both browser and Node/SSR contexts. */
export const publicApiBase = (): string => env.PUBLIC_API_URL || 'http://localhost:8080';

/** Shorthand alias. */
export const apiBase = publicApiBase;
