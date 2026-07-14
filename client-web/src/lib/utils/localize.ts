import { getLocale } from '$lib/paraglide/runtime.js';

export function getLocalizedField<T extends Record<string, any>>(
  entity: T,
  baseField: string,
  locale: string
): string {
  if (!entity) return '';

  const fieldsToTry = [baseField];
  if (!baseField) {
    fieldsToTry.push('name');
    fieldsToTry.push('title');
  }

  for (const field of fieldsToTry) {
    // 1. Try camelCase (e.g. nameAr, nameEn)
    const suffixCamel = locale === 'ar' ? 'Ar' : 'En';
    const keyCamel = field ? `${field}${suffixCamel}` : (locale === 'ar' ? 'Ar' : 'En');
    if (entity[keyCamel] !== undefined && entity[keyCamel] !== null) return String(entity[keyCamel]);
    
    // 2. Try snake_case (e.g. name_ar, name_en)
    const suffixSnake = locale === 'ar' ? '_ar' : '_en';
    const keySnake = field ? `${field}${suffixSnake}` : (locale === 'ar' ? '_ar' : '_en');
    if (entity[keySnake] !== undefined && entity[keySnake] !== null) return String(entity[keySnake]);

    // 3. Try direct locale code (e.g. ar, en)
    if (entity[locale] !== undefined && entity[locale] !== null) return String(entity[locale]);
    
    // 4. Try camelCase fallback
    const fallbackSuffixCamel = locale === 'ar' ? 'En' : 'Ar';
    const fallbackKeyCamel = field ? `${field}${fallbackSuffixCamel}` : (locale === 'ar' ? 'En' : 'Ar');
    if (entity[fallbackKeyCamel] !== undefined && entity[fallbackKeyCamel] !== null) return String(entity[fallbackKeyCamel]);

    // 5. Try snake_case fallback
    const fallbackSuffixSnake = locale === 'ar' ? '_en' : '_ar';
    const fallbackKeySnake = field ? `${field}${fallbackSuffixSnake}` : (locale === 'ar' ? '_en' : '_ar');
    if (entity[fallbackKeySnake] !== undefined && entity[fallbackKeySnake] !== null) return String(entity[fallbackKeySnake]);

    // 6. Try direct locale fallback
    const fallbackLocale = locale === 'ar' ? 'en' : 'ar';
    if (entity[fallbackLocale] !== undefined && entity[fallbackLocale] !== null) return String(entity[fallbackLocale]);

    // 7. Base field fallback
    if (field && entity[field] !== undefined && entity[field] !== null) return String(entity[field]);
  }
  
  return '';
}

export function formatCurrency(value: number, locale: string = getLocale(), currency: string = 'SAR'): string {
  return new Intl.NumberFormat(locale === 'ar' ? 'ar-SA' : 'en-US', { style: 'currency', currency }).format(value);
}

export function formatDate(date: string | Date, locale: string = getLocale()): string {
  if (!date) return '';
  try {
    const d = new Date(date);
    if (isNaN(d.getTime()) || d.getFullYear() < 1000) {
      return '';
    }
    return new Intl.DateTimeFormat(locale === 'ar' ? 'ar-SA' : 'en-US').format(d);
  } catch (e) {
    return '';
  }
}

export function formatNumber(value: number, locale: string = getLocale()): string {
  return new Intl.NumberFormat(locale === 'ar' ? 'ar-SA' : 'en-US').format(value);
}
