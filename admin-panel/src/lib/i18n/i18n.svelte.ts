import { t as tStore, lang, dir, isRTL } from './index';

// A simple mock for I18nService to satisfy UnifiedDescriptionBuilder's `getI18n()` and wizard components
export class I18nService {
  get locale() {
    let currentLang = 'ar';
    lang.subscribe(l => { currentLang = l; })();
    return currentLang;
  }

  get t() {
    let translateFn: (key: string, fallback?: string) => string = (k, f) => f || k;
    tStore.subscribe(fn => { translateFn = fn; })();
    return translateFn;
  }
}

const mockService = new I18nService();

export function getI18n() {
  return mockService;
}
