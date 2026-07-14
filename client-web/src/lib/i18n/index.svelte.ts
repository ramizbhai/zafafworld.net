import { browser } from '$app/environment';
import { ar } from './translations/ar.js';
import { en } from './translations/en.js';
import { DEFAULT_LOCALE, LOCALE_STORAGE_KEY, getLocaleConfig } from './config/index.js';
import type { Locale, Direction } from './types.js';
import type { Translations } from './translations/ar.js';

// ── Translation registry ──────────────────────────────────────────────────
const TRANSLATIONS: Record<Locale, Translations> = { ar, en };

// ── Svelte 5 Rune-based i18n store ───────────────────────────────────────
function createI18n() {
  const storedLocale = browser
    ? (localStorage.getItem(LOCALE_STORAGE_KEY) as Locale | null)
    : null;

  let locale = $state<Locale>(storedLocale ?? DEFAULT_LOCALE);

  const config = $derived(getLocaleConfig(locale));
  const t = $derived(TRANSLATIONS[locale]);
  const dir = $derived<Direction>(config.direction);
  const isRTL = $derived(dir === 'rtl');

  function setLocale(newLocale: Locale) {
    locale = newLocale;
    if (browser) {
      localStorage.setItem(LOCALE_STORAGE_KEY, newLocale);
      // Update document-level lang + dir
      document.documentElement.lang = newLocale;
      document.documentElement.dir = getLocaleConfig(newLocale).direction;
    }
  }

  function toggleLocale() {
    setLocale(locale === 'ar' ? 'en' : 'ar');
  }

  /**
   * Interpolate translation with dynamic values.
   * Usage: interpolate(t.errors.minLength, { min: 8 })
   */
  function interpolate(template: string, values: Record<string, string | number>): string {
    return Object.entries(values).reduce(
      (str, [key, val]) => str.replaceAll(`{${key}}`, String(val)),
      template,
    );
  }

  /**
   * Format a number according to locale.
   */
  function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
    return new Intl.NumberFormat(config.numberLocale, options).format(value);
  }

  /**
   * Format currency in SAR.
   */
  function formatCurrency(value: number): string {
    if (locale === 'ar') {
      return `${formatNumber(value)} ${t.common.currency}`;
    }
    return `${t.common.currency} ${formatNumber(value)}`;
  }

  /**
   * Format a date according to locale.
   */
  function formatDate(date: Date | string, options?: Intl.DateTimeFormatOptions): string {
    const d = typeof date === 'string' ? new Date(date) : date;
    return new Intl.DateTimeFormat(config.dateLocale, options).format(d);
  }

  return {
    get locale() { return locale; },
    get config() { return config; },
    get t() { return t; },
    get dir() { return dir; },
    get isRTL() { return isRTL; },
    setLocale,
    toggleLocale,
    interpolate,
    formatNumber,
    formatCurrency,
    formatDate,
  };
}

export const i18n = createI18n();
