<script lang="ts">
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { Sparkles, Check } from 'lucide-svelte';
  import { featuresStore } from '$lib/stores/features.svelte.js';

  let { featuresSelection = {}, culturalAttributes = {} } = $props<{
    featuresSelection?: Record<string, any>;
    culturalAttributes?: Record<string, any>;
  }>();

  const isAr = $derived(getLocale() === 'ar');

  // Known attribute dictionary for legacy/snake_case keys & cultural amenities
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

  const activeFeatures = $derived.by(() => {
    const list: { id: string; label: string; value?: string }[] = [];

    // Helper to resolve label from master store or dictionary
    const resolveLabel = (key: string): string => {
      // 1. Check master DB featuresStore (UUID lookup)
      if (featuresStore.map[key]) {
        const feat = featuresStore.map[key];
        return isAr ? feat.nameAr : feat.nameEn;
      }
      // 2. Check local cultural / legacy dictionary
      if (culturalDict[key]) {
        return isAr ? culturalDict[key].ar : culturalDict[key].en;
      }
      // 3. Fallback clean up formatted key
      const formatted = key.replace(/_/g, ' ');
      return formatted.charAt(0).toUpperCase() + formatted.slice(1);
    };

    // Process cultural attributes
    if (culturalAttributes && typeof culturalAttributes === 'object') {
      for (const [key, val] of Object.entries(culturalAttributes)) {
        if (val === null || val === undefined || val === false || val === '') continue;
        const label = resolveLabel(key);
        if (typeof val === 'boolean' && val) {
          list.push({ id: key, label });
        } else if (typeof val === 'string' || typeof val === 'number') {
          list.push({ id: key, label, value: String(val) });
        }
      }
    }

    // Process features selection (JSON dynamic entries)
    if (featuresSelection && typeof featuresSelection === 'object') {
      for (const [key, val] of Object.entries(featuresSelection)) {
        if (val === null || val === undefined || val === false || val === '') continue;
        
        // If already added, skip duplicate
        if (list.some(i => i.id === key)) continue;

        const label = resolveLabel(key);
        if (typeof val === 'boolean' && val) {
          list.push({ id: key, label });
        } else if (typeof val === 'string' || typeof val === 'number') {
          list.push({ id: key, label, value: String(val) });
        }
      }
    }

    return list;
  });
</script>

{#if activeFeatures.length > 0}
  <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm hover:shadow-md transition-shadow">
    <div class="flex items-center gap-3 mb-6 border-b border-slate-100 pb-4">
      <div class="p-2.5 bg-blue-50 text-blue-600 rounded-xl">
        <Sparkles size={22} />
      </div>
      <h3 class="font-display text-xl font-bold text-slate-900">
        {isAr ? 'الميزات والخدمات المضافة' : 'Features & Added Services'}
      </h3>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
      {#each activeFeatures as feat}
        <div class="flex items-center gap-3 p-3.5 rounded-xl bg-slate-50/70 border border-slate-100/80 transition-colors hover:bg-slate-100/60">
          <div class="w-6 h-6 rounded-full bg-blue-100 text-blue-700 flex items-center justify-center shrink-0">
            <Check size={14} class="stroke-[3]" />
          </div>
          <div class="flex flex-col">
            <span class="text-xs sm:text-sm font-semibold text-slate-800">{feat.label}</span>
            {#if feat.value}
              <span class="text-xs font-bold text-blue-600">{feat.value}</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>
{/if}
