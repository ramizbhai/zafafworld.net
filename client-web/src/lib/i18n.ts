import { createI18n } from "@inlang/paraglide-sveltekit";
import * as runtime from "$lib/paraglide/runtime.js";

// @ts-ignore — bridge v1 runtime shape to createI18n's expected interface
const r = runtime as any;
const runtimeWrapper = {
    ...r,
    availableLanguageTags: r.locales ?? r.availableLanguageTags,
    sourceLanguageTag: (r.baseLocale ?? r.sourceLanguageTag) as "ar" | "en",
    languageTag: r.getLocale ?? r.languageTag,
    setLanguageTag: r.setLocale ?? r.setLanguageTag,
    isAvailableLanguageTag: r.isLocale ?? r.isAvailableLanguageTag,
};

// @ts-ignore
export const i18n: any = createI18n(runtimeWrapper, {
    defaultLanguageTag: "ar",
    exclude: [/^\/api/, /^\/bff/, /^\/healthz/],
    prefixDefaultLanguage: "always"
});
