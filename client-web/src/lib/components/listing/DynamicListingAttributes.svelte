<script lang="ts">
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { Users, ShieldCheck, Clock, Layers, Award, CheckCircle2 } from 'lucide-svelte';
  import { featuresStore } from '$lib/stores/features.svelte.js';

  let { attributes = {}, category = '' } = $props<{
    attributes: Record<string, any>;
    category?: string;
  }>();

  const isAr = $derived(getLocale() === 'ar');

  // Attribute dictionary for bilingual labels and units
  const attributeDictionary: Record<string, { ar: string; en: string; icon?: any; type?: 'boolean' | 'number' | 'text'; unitAr?: string; unitEn?: string }> = {
    men_capacity: { ar: 'سعة قاعة الرجال', en: "Men's Hall Capacity", type: 'number', unitAr: 'شخص', unitEn: 'guests' },
    women_capacity: { ar: 'سعة قاعة النساء', en: "Women's Hall Capacity", type: 'number', unitAr: 'شخص', unitEn: 'guests' },
    has_separate_entrances: { ar: 'مداخل منفصلة للرجال والنساء', en: 'Separate Entrances for Men & Women', type: 'boolean' },
    has_audio_link: { ar: 'ربط صوتي بين القاعات', en: 'Audio Link Between Halls', type: 'boolean' },
    max_events_per_day: { ar: 'حجز حصري (حدث واحد يومياً)', en: 'Exclusive (1 Event Per Day)', type: 'boolean' },
    weekend_surcharge_sar: { ar: 'رسوم إضافية لنهائية الأسبوع', en: 'Weekend Surcharge', type: 'number', unitAr: 'ر.س', unitEn: 'SAR' },
    private_pool: { ar: 'مسبح خاص متوفر', en: 'Private Pool Available', type: 'boolean' },
    in_house_catering: { ar: 'خدمة ضيافة وطعام داخلية', en: 'In-house Catering Provided', type: 'boolean' },
    private_hall_available: { ar: 'قاعة خاصة متوفرة', en: 'Private Hall Available', type: 'boolean' },
    family_section: { ar: 'قسم عوائل خاص', en: 'Family Section Available', type: 'boolean' },
    team_size: { ar: 'حجم فريق العمل', en: 'Team Size', type: 'number', unitAr: 'أفراد', unitEn: 'members' },
    delivery_weeks: { ar: 'مدة التسليم', en: 'Delivery Time', type: 'number', unitAr: 'أسابيع', unitEn: 'weeks' },
    female_team_available: { ar: 'طاقم عمل نسائي متوفر', en: 'Female Team Available', type: 'boolean' },
    women_section_coverage: { ar: 'تغطية كاملة لقسم النساء', en: 'Women Section Coverage', type: 'boolean' },
    drone_available: { ar: 'تصوير طيران (درون)', en: 'Drone Photography Available', type: 'boolean' },
    highlight_reel: { ar: 'فيديو ملخص (هايلايت)', en: 'Highlight Reel Included', type: 'boolean' },
    min_guests: { ar: 'الحد الأدنى للضيوف', en: 'Minimum Guests Required', type: 'number', unitAr: 'شخص', unitEn: 'guests' },
    buffet_or_plated: { ar: 'نوع الخدمة', en: 'Service Type', type: 'text' },
    halal_certified: { ar: 'شهادة حلال', en: 'Halal Certified', type: 'boolean' },
    serving_staff_included: { ar: 'طاقم الخدمة متضمن', en: 'Serving Staff Included', type: 'boolean' },
    taste_testing: { ar: 'جلسة تذوق متوفرة', en: 'Taste Testing Session Available', type: 'boolean' },
    setup_cleanup: { ar: 'التجهيز والتنظيف متضمن', en: 'Setup & Cleanup Included', type: 'boolean' }
  };

  // Filter only attributes that exist and have non-null values
  const validAttributes = $derived.by(() => {
    if (!attributes || typeof attributes !== 'object') return [];
    
    const items: { key: string; label: string; valueDisplay: string; isBoolean: boolean }[] = [];

    for (const [key, val] of Object.entries(attributes)) {
      if (val === null || val === undefined || val === '') continue;
      
      let label = key.replace(/_/g, ' ');
      const config = attributeDictionary[key];
      
      if (featuresStore.map[key]) {
        const feat = featuresStore.map[key];
        label = isAr ? feat.nameAr : feat.nameEn;
      } else if (config) {
        label = isAr ? config.ar : config.en;
      }
      
      if (typeof val === 'boolean') {
        if (val) {
          items.push({ key, label, valueDisplay: isAr ? 'متوفر' : 'Available', isBoolean: true });
        }
      } else if (typeof val === 'number' || !isNaN(Number(val))) {
        const numVal = Number(val);
        if (numVal > 0) {
          const unit = config ? (isAr ? (config.unitAr || '') : (config.unitEn || '')) : '';
          items.push({ key, label, valueDisplay: `${numVal.toLocaleString()} ${unit}`.trim(), isBoolean: false });
        }
      } else if (typeof val === 'string') {
        let textDisplay = val;
        if (key === 'buffet_or_plated') {
          if (val === 'buffet') textDisplay = isAr ? 'بوفيه مفتوح' : 'Open Buffet';
          else if (val === 'plated') textDisplay = isAr ? 'خدمة أطباق' : 'Plated Service';
          else if (val === 'both') textDisplay = isAr ? 'كلا الخيارين' : 'Both Options';
        }
        items.push({ key, label, valueDisplay: textDisplay, isBoolean: false });
      }
    }
    return items;
  });
</script>

{#if validAttributes.length > 0}
  <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm hover:shadow-md transition-shadow">
    <div class="flex items-center gap-3 mb-6 border-b border-slate-100 pb-4">
      <div class="p-2.5 bg-amber-50 text-amber-600 rounded-xl">
        <Award size={22} />
      </div>
      <h3 class="font-display text-xl font-bold text-slate-900">
        {isAr ? 'المواصفات والتفاصيل الخاصة' : 'Specific Specifications & Details'}
      </h3>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each validAttributes as item}
        <div class="flex items-center justify-between p-4 rounded-xl bg-slate-50/80 border border-slate-100/80">
          <div class="flex items-center gap-3">
            <CheckCircle2 size={18} class="text-amber-500 shrink-0" />
            <span class="text-sm font-medium text-slate-700">{item.label}</span>
          </div>
          {#if !item.isBoolean || item.valueDisplay !== (isAr ? 'متوفر' : 'Available')}
            <span class="text-sm font-bold text-slate-900 bg-white px-3 py-1 rounded-lg border border-slate-200/60 shadow-2xs">
              {item.valueDisplay}
            </span>
          {:else}
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200/50">
              ✓ {item.valueDisplay}
            </span>
          {/if}
        </div>
      {/each}
    </div>
  </section>
{/if}
