<script lang="ts">
  import {
    CheckCircle2,
    MapPin,
    Users,
    Building,
    ShieldCheck,
    Star,
  } from "lucide-svelte";
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { featuresStore } from '$lib/stores/features.svelte.js';

  // The raw JSON attributes object from the backend
  let { attributes = {} } = $props<{ attributes: Record<string, any> }>();

  import * as m from "$lib/paraglide/messages.js";

  const isAr = $derived(getLocale() === 'ar');

  const culturalDict: Record<string, { ar: string; en: string }> = {
    prayer_room: { ar: 'مصلى للرجال والنساء', en: 'Prayer Room / Musala' },
    valet_parking: { ar: 'خدمة صف السيارات (فالياه)', en: 'Valet Parking' },
    bridal_suite: { ar: 'جناح خاص بالعروس', en: 'Bridal Suite' },
    outdoor_garden: { ar: 'حديقة وجلسات خارجية', en: 'Outdoor Garden & Seating' },
    external_catering_allowed: { ar: 'السماح بالضيافة الخارجية', en: 'External Catering Allowed' },
    halal_certified: { ar: 'شهادة حلال موثقة', en: 'Halal Certified' },
    has_audio_link: { ar: 'ربط صوتي بين القاعات', en: 'Audio Link Between Halls' },
    has_separate_entrances: { ar: 'مداخل منفصلة للرجال والنساء', en: 'Separate Entrances for Men & Women' },
    max_events_per_day: { ar: 'حجز حصري (حدث واحد يومياً)', en: 'Exclusive (1 Event Per Day)' },
    men_capacity: { ar: 'سعة قاعة الرجال', en: "Men's Hall Capacity" },
    women_capacity: { ar: 'سعة قاعة النساء', en: "Women's Hall Capacity" },
    private_pool: { ar: 'مسبح خاص متوفر', en: 'Private Pool Available' },
    in_house_catering: { ar: 'خدمة ضيافة وطعام داخلية', en: 'In-house Catering Provided' },
    private_hall_available: { ar: 'قاعة خاصة متوفرة', en: 'Private Hall Available' },
    family_section: { ar: 'قسم عوائل خاص', en: 'Family Section Available' },
    team_size: { ar: 'حجم فريق العمل', en: 'Team Size' },
    delivery_weeks: { ar: 'مدة التسليم', en: 'Delivery Time' },
    female_team_available: { ar: 'طاقم عمل نسائي متوفر', en: 'Female Team Available' },
    women_section_coverage: { ar: 'تغطية كاملة لقسم النساء', en: 'Women Section Coverage' },
    drone_available: { ar: 'تصوير طيران (درون)', en: 'Drone Photography Available' },
    highlight_reel: { ar: 'فيديو ملخص (هايلايت)', en: 'Highlight Reel Included' },
    min_guests: { ar: 'الحد الأدنى للضيوف', en: 'Minimum Guests Required' },
    serving_staff_included: { ar: 'طاقم الخدمة متضمن', en: 'Serving Staff Included' },
    taste_testing: { ar: 'جلسة تذوق متوفرة', en: 'Taste Testing Session Available' },
    setup_cleanup: { ar: 'التجهيز والتنظيف متضمن', en: 'Setup & Cleanup Included' }
  };

  // Helper to format key to bilingual title
  function formatLabel(key: string) {
    if (featuresStore.map[key]) {
      const feat = featuresStore.map[key];
      return isAr ? feat.nameAr : feat.nameEn;
    }
    if (culturalDict[key]) {
      return isAr ? culturalDict[key].ar : culturalDict[key].en;
    }
    const attrKey = `attr_${key}` as keyof typeof m;
    if (m[attrKey] && typeof m[attrKey] === 'function') {
      return (m[attrKey] as () => string)();
    }
    const formatted = key.replace(/_/g, " ");
    return formatted.charAt(0).toUpperCase() + formatted.slice(1);
  }

  // Filter out false, null, empty strings
  const activeFeatures = $derived(
    Object.entries(attributes).filter(
      ([_, val]) => val === true || (typeof val === "string" && val.length > 0),
    ),
  );

  // Map certain keys to specific icons, default to CheckCircle2
  function getIconFor(key: string) {
    if (key.includes("parking") || key.includes("outdoor")) return MapPin;
    if (
      key.includes("capacity") ||
      key.includes("worker") ||
      key.includes("supervisor")
    )
      return Users;
    if (key.includes("room") || key.includes("hall") || key.includes("suite"))
      return Building;
    if (key.includes("inspector")) return ShieldCheck;
    if (key.includes("featured") || key.includes("vip")) return Star;
    return CheckCircle2;
  }
</script>

<div class="bg-white rounded-2xl border border-gray-100 p-8 shadow-sm">
  <h3
    class="font-display text-2xl font-semibold text-slate-800 mb-6 border-b pb-4"
  >
    {m.listing_premium_features_title()}
  </h3>

  {#if activeFeatures.length > 0}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each activeFeatures as [key, val]}
        {@const Icon = getIconFor(key)}
        <div class="flex items-start gap-3 group">
          <div
            class="p-2 rounded-lg bg-slate-50 group-hover:bg-amber-50 text-slate-400 group-hover:text-amber-600 transition-colors"
          >
            <Icon size={20} strokeWidth={2.5} />
          </div>
          <div>
            <p class="font-medium text-slate-700 leading-tight pt-1.5">
              {formatLabel(key)}
            </p>
            {#if typeof val === "string"}
              <p class="text-sm text-slate-500 mt-1">{val}</p>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <!-- Auto-collapse if empty, but we show a small fallback if literally zero features to avoid broken UI feeling -->
    <p class="text-slate-500 italic">
      {m.listing_premium_features_empty()}
    </p>
  {/if}
</div>
