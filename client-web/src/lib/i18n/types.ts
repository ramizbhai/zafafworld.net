export type Locale = 'ar' | 'en';
export type Direction = 'rtl' | 'ltr';

export interface LocaleConfig {
  code: Locale;
  name: string;
  nativeName: string;
  direction: Direction;
  dateLocale: string;
  numberLocale: string;
}

export interface TranslationDictionary {
  [key: string]: string | TranslationDictionary;
}

export type TranslationKey = string;
