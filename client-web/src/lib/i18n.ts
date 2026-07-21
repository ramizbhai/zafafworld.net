import { createI18n } from "@inlang/paraglide-sveltekit";
import * as runtime from "$lib/paraglide/runtime.js";

// @ts-ignore
const r = runtime as any;
const runtimeWrapper = {
    ...r,
    availableLanguageTags: r.locales ?? r.availableLanguageTags,
    sourceLanguageTag: (r.baseLocale ?? r.sourceLanguageTag) as "ar" | "en",
    languageTag: r.getLocale ?? r.languageTag,
    setLanguageTag: r.setLocale ?? r.setLanguageTag,
    isAvailableLanguageTag: r.isLocale ?? r.isAvailableLanguageTag,
};

// Create the Paraglide-SvelteKit i18n instance
// @ts-ignore
export const i18n: any = createI18n(runtimeWrapper, {
    defaultLanguageTag: "ar",
    exclude: [/^\/api/, /^\/bff/, /^\/healthz/],
    prefixDefaultLanguage: "always"
});

// Override getLocale to bridge Paraglide v2 runtime with SvelteKit URL routing & AsyncLocalStorage
if (r.overwriteGetLocale) {
    const originalGetLocale = r.getLocale;
    r.overwriteGetLocale(() => {
        // 1. Server-side check (Paraglide-SvelteKit AsyncLocalStorage hook)
        if (typeof runtimeWrapper.setLanguageTag === 'function' && runtimeWrapper.setLanguageTag.length === 0) {
            return runtimeWrapper.setLanguageTag();
        }
        // 2. Client-side browser check: derive language directly from active URL
        if (typeof window !== 'undefined' && window.location?.href) {
            try {
                const urlLang = i18n.getLanguageFromUrl(new URL(window.location.href));
                if (urlLang === 'ar' || urlLang === 'en') {
                    return urlLang;
                }
            } catch {
                // Ignore URL parsing errors
            }
        }
        // 3. Fallback to default runtime logic
        return originalGetLocale();
    });
}

