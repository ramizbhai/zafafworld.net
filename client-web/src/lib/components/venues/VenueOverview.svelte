<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";

  let { venue, description, t } = $props<{
    venue: any;
    description: string;
    t: (ar: string, en: string) => string;
  }>();
</script>

<div
  class="bg-white p-6 md:p-8 rounded-2xl border border-[var(--color-border)] shadow-sm text-start"
>
  <h2
    class="font-display text-xl md:text-2xl font-bold text-[var(--color-secondary)] mb-6 underline-gold pb-1"
  >
    {t("عن مزوّد الخدمة", "About Vendor")}
  </h2>
  <div
    class="prose max-w-none text-[var(--color-text)] leading-relaxed text-sm md:text-base space-y-4"
  >
    <p class="whitespace-pre-line">
      {description ||
        t(
          "لا يوجد وصف متاح باللغة المحددة حالياً.",
          "No description available in this language yet.",
        )}
    </p>
  </div>

  <!-- General Business Highlights -->
  <div
    class="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-8 pt-6 border-t border-[var(--color-border)] text-sm"
  >
    <div class="flex items-center gap-3">
      <span class="text-xl">🎓</span>
      <div>
        <span class="block text-xs text-[var(--color-muted)]"
          >{t("الخبرة والمهارة", "Experience")}</span
        >
        <span class="font-bold text-[var(--color-secondary)]"
          >{t(
            "٥+ سنوات من الاحتراف",
            "5+ Years of Excellence",
          )}</span
        >
      </div>
    </div>

    <div class="flex items-center gap-3">
      <span class="text-xl">🗣️</span>
      <div>
        <span class="block text-xs text-[var(--color-muted)]"
          >{t("اللغات المتاحة", "Languages Spoken")}</span
        >
        <span class="font-bold text-[var(--color-secondary)]"
          >{t("العربية، الإنجليزية", "Arabic, English")}</span
        >
      </div>
    </div>

    <div class="flex items-center gap-3">
      <span class="text-xl">🗺️</span>
      <div>
        <span class="block text-xs text-[var(--color-muted)]"
          >{t(
            "مناطق الخدمة المشمولة",
            "Service Areas Covered",
          )}</span
        >
        <span class="font-bold text-[var(--color-secondary)]">
          {venue.location.city}
          {getLocale() === "ar"
            ? "وكافة ضواحيها"
            : "& surrounding areas"}
        </span>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <span class="text-xl">⏱️</span>
      <div>
        <span class="block text-xs text-[var(--color-muted)]"
          >{t("سرعة الرد والاستجابة", "Response Speed")}</span
        >
        <span class="font-bold text-emerald-600"
          >✓ {t(
            "يرد في أقل من ساعة واحدة",
            "Usually replies within 1 hour",
          )}</span
        >
      </div>
    </div>
  </div>

  <!-- Included amenities / pricing options -->
  {#if venue.pricing?.includedServices && venue.pricing.includedServices.length > 0}
    <div class="mt-8 pt-6 border-t border-[var(--color-border)]">
      <h3
        class="font-display text-base font-bold text-[var(--color-secondary)] mb-4"
      >
        {m.auto_included_services()}
      </h3>
      <div class="flex flex-wrap gap-2">
        {#each venue.pricing.includedServices as service}
          <span
            class="text-xs px-3 py-1.5 bg-emerald-50 text-emerald-700 border border-emerald-100 rounded-lg font-bold flex items-center gap-1"
          >
            ✓ {service}
          </span>
        {/each}
      </div>
    </div>
  {/if}
</div>
