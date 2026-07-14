import type { LocaleConfig, Locale } from '../types.js';

export const DEFAULT_LOCALE: Locale = 'ar';

export const SUPPORTED_LOCALES: LocaleConfig[] = [
  {
    code: 'ar',
    name: 'Arabic',
    nativeName: 'العربية',
    direction: 'rtl',
    dateLocale: 'ar-SA',
    numberLocale: 'ar-SA',
  },
  {
    code: 'en',
    name: 'English',
    nativeName: 'English',
    direction: 'ltr',
    dateLocale: 'en-US',
    numberLocale: 'en-US',
  },
];

export const LOCALE_STORAGE_KEY = 'zefaf_locale';

export function getLocaleConfig(code: Locale): LocaleConfig {
  return SUPPORTED_LOCALES.find((l) => l.code === code) ?? SUPPORTED_LOCALES[0];
}
