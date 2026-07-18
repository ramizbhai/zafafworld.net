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

if (r.overwriteGetLocale) {
    const originalGetLocale = r.getLocale;
    r.overwriteGetLocale(() => {
        // The paraglide-sveltekit adapter overrides runtimeWrapper.setLanguageTag
        // with a 0-argument function that GETS the current locale from its AsyncLocalStorage.
        // We detect this and use it as the source of truth for the v2 runtime.
        if (typeof runtimeWrapper.setLanguageTag === 'function' && runtimeWrapper.setLanguageTag.length === 0) {
            return runtimeWrapper.setLanguageTag();
        }
        return originalGetLocale();
    });
}

// @ts-ignore
export const i18n: any = createI18n(runtimeWrapper, {
    defaultLanguageTag: "ar",
    exclude: [/^\/api/, /^\/bff/, /^\/healthz/],
    prefixDefaultLanguage: "always"
});
