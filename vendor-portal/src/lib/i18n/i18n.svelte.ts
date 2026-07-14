import { browser } from '$app/environment';
import { getContext, setContext } from 'svelte';
import { ar } from './locales/ar';
import { en } from './locales/en';

export type Locale = 'ar' | 'en';
export type TranslationDict = typeof ar & ((key: string, params?: Record<string, string | number>) => string);

function createTranslationProxy(locale: Locale, dict: any, service: I18nService): any {
  const translateFn = (path: string, params?: Record<string, string | number>) => {
    let val = path.split('.').reduce((acc, part) => acc && acc[part], dict);
    if (typeof val !== 'string') {
      return path;
    }
    if (params) {
      val = service.interpolate(val, params);
    }
    return val;
  };

  return new Proxy(translateFn, {
    get(target, prop, receiver) {
      if (typeof prop === 'string') {
        const val = dict[prop];
        if (val && typeof val === 'object') {
          return createTranslationProxy(locale, val, service);
        }
        return val;
      }
      return Reflect.get(target, prop, receiver);
    }
  });
}

export class I18nService {
  #locale = $state<Locale>('ar');

  constructor(initialLocale?: Locale) {
    if (initialLocale) {
      this.#locale = initialLocale;
    } else if (browser) {
      const match = document.cookie.match(/(?:^|; )zafaf_locale=([^;]+)/);
      if (match && (match[1] === 'ar' || match[1] === 'en')) {
        this.#locale = match[1] as Locale;
      }
    }
    
    if (browser) {
      this.updateHtmlAttributes(this.#locale);
    }
  }

  get locale(): Locale {
    return this.#locale;
  }

  set locale(newLocale: Locale) {
    if (newLocale === 'ar' || newLocale === 'en') {
      this.#locale = newLocale;
      if (browser) {
        document.cookie = `zafaf_locale=${newLocale}; path=/; max-age=31536000`;
        this.updateHtmlAttributes(newLocale);
      }
    }
  }

  get t(): TranslationDict {
    return createTranslationProxy(this.#locale, this.#locale === 'ar' ? ar : en, this);
  }

  get isRtl(): boolean {
    return this.#locale === 'ar';
  }

  updateHtmlAttributes(locale: Locale) {
    if (browser) {
      const dir = locale === 'ar' ? 'rtl' : 'ltr';
      document.documentElement.dir = dir;
      document.documentElement.lang = locale;
    }
  }

  interpolate(str: string, params: Record<string, string | number>): string {
    let result = str;
    for (const [key, value] of Object.entries(params)) {
      result = result.replace(new RegExp(`{${key}}`, 'g'), String(value));
    }
    return result;
  }
}

const I18N_KEY = Symbol('i18n');

// Standard fallback instance for static/client only use
export const i18n = new I18nService();

export function setI18nContext(initialLocale: Locale) {
  const service = new I18nService(initialLocale);
  setContext(I18N_KEY, service);
  return service;
}

export function getI18n(): I18nService {
  const service = getContext<I18nService>(I18N_KEY);
  return service || i18n;
}
