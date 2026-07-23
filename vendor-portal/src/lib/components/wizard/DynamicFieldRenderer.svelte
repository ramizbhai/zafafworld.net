<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { listingStore } from '$lib/stores/listingStore';

    const i18n = getI18n();
    let { fieldId, required = false, visibleWhen = null } = $props<{ fieldId: string, required?: boolean, visibleWhen?: string | null }>();

    // safe evaluation of visibleWhen condition against listingStore data
    const isVisible = $derived.by(() => {
        if (!visibleWhen) return true;

        const parts = visibleWhen.split(/\s+/);
        if (parts.length === 3) {
            const field = parts[0];
            const op = parts[1];
            let valStr = parts[2];
            
            if ((valStr.startsWith("'") && valStr.endsWith("'")) || (valStr.startsWith('"') && valStr.endsWith('"'))) {
                valStr = valStr.substring(1, valStr.length - 1);
            }

            let currentVal = ($listingStore.formData as any)[field];
            if (currentVal === undefined) {
                currentVal = ($listingStore.formData.categoryAttributes as any)[field];
            }
            if (currentVal === undefined) {
                currentVal = ($listingStore.formData.culturalAttributes as any)[field];
            }

            let compareVal: any = valStr;
            if (valStr === 'true') compareVal = true;
            else if (valStr === 'false') compareVal = false;
            else if (!isNaN(Number(valStr))) compareVal = Number(valStr);

            if (op === '===') {
                return currentVal === compareVal;
            } else if (op === '!==') {
                return currentVal !== compareVal;
            }
        }
        return true;
    });

    // Static dictionary of bilingual labels
    const FIELD_LABELS: Record<string, { en: string, ar: string }> = {
        men_capacity: { en: "Men Capacity", ar: "سعة قسم الرجال" },
        women_capacity: { en: "Women Capacity", ar: "سعة قسم النساء" },
        has_separate_entrances: { en: "Separate Entrances for Men & Women", ar: "مداخل منفصلة للرجال والنساء" },
        has_audio_link: { en: "Audio Link between Sections", ar: "ربط صوتي بين الأقسام" },
        max_events_per_day: { en: "Max Events per Day", ar: "الحد الأقصى للمناسبات في اليوم" },
        weekend_surcharge_sar: { en: "Weekend Surcharge (SAR)", ar: "رسوم إضافية لنهاية الأسبوع (ريال)" },
        private_pool: { en: "Private Pool Available", ar: "مسبح خاص متوفر" },
        in_house_catering: { en: "In-house Catering Available", ar: "بوفيه داخلي متوفر" },
        private_hall_available: { en: "Private Hall Available", ar: "قاعة خاصة متوفرة" },
        family_section: { en: "Family Section Available", ar: "قسم عائلي متوفر" },
        team_size: { en: "Number of staff", ar: "عدد الموظفين" },
        delivery_weeks: { en: "Delivery Time (Weeks)", ar: "مدة التسليم (بالأسابيع)" },
        women_section_coverage: { en: "Women Section Coverage Only", ar: "تغطية قسم النساء فقط" },
        drone_available: { en: "Drone Photography Available", ar: "تصوير طائرة (درون) متوفر" },
        highlight_reel: { en: "Highlight Reel Included", ar: "فيديو ملخص متضمن" },
        min_guests: { en: "Minimum Guests", ar: "الحد الأدنى للضيوف" },
        buffet_or_plated: { en: "Buffet or Plated Service", ar: "نوع الخدمة (بوفيه / تقديم أطباق)" },
        halal_certified: { en: "Halal Certified Menu", ar: "قائمة طعام معتمدة حلال" },
        taste_testing: { en: "Taste Testing Session Available", ar: "جلسة تذوق متوفرة" },
        setup_cleanup: { en: "Setup & Cleanup Included", ar: "التجهيز والتنظيف متضمن" },
        rehearsal_count: { en: "Rehearsal Sessions", ar: "عدد جلسات البروفة" },
        tailoring_time_days: { en: "Tailoring Time (Days)", ar: "مدة الخياطة (بالأيام)" },
        bride_companions_count: { en: "Bride Companions Count", ar: "عدد مرافقات العروس" },
        address: { en: "Business Address", ar: "عنوان مقر العمل" },
        events_hosted_description: { en: "Events Hosted Description", ar: "وصف المناسبات المقدمة" },
        preparation_time_hours: { en: "Preparation Time (Hours)", ar: "مدة التحضير (بالساعات)" },
        vehicle_count: { en: "Number of Vehicles", ar: "عدد السيارات المتوفرة" },
        capacity: { en: "Total Capacity", ar: "إجمالي الطاقة الاستيعابية" },
        elevator_access: { en: "Elevator Access", ar: "مصعد متوفر" },
        wind_protection: { en: "Wind Protection / Shields", ar: "حواجز مضادة للرياح" },
        city_view: { en: "City View", ar: "إطلالة على المدينة" },
        noise_curfew_time: { en: "Noise Curfew Time", ar: "وقت حظر الموسيقى/الضوضاء" },
        private_or_semi: { en: "Privacy Type", ar: "نوع الخصوصية (خاص / شبه خاص)" },
        floating_stage: { en: "Floating Stage Available", ar: "منصة عائمة متوفرة" },
        shade_structures: { en: "Shade / Canopy Structures", ar: "مظلات وهياكل ظل" }
    };

    const label = $derived(
        i18n.locale === 'ar'
            ? FIELD_LABELS[fieldId]?.ar || fieldId
            : FIELD_LABELS[fieldId]?.en || fieldId
    );

    // Identify field input type
    const isBoolean = $derived([
        'has_separate_entrances',
        'has_audio_link',
        'private_pool',
        'in_house_catering',
        'private_hall_available',
        'family_section',
        'women_section_coverage',
        'drone_available',
        'highlight_reel',
        'halal_certified',
        'taste_testing',
        'setup_cleanup',
        'elevator_access',
        'wind_protection',
        'city_view',
        'floating_stage',
        'shade_structures'
    ].includes(fieldId));

    const isSelect = $derived(fieldId === 'buffet_or_plated');
    const isTextarea = $derived(fieldId === 'events_hosted_description');
    const isNumber = $derived([
        'men_capacity',
        'women_capacity',
        'max_events_per_day',
        'weekend_surcharge_sar',
        'team_size',
        'delivery_weeks',
        'min_guests',
        'rehearsal_count',
        'tailoring_time_days',
        'bride_companions_count',
        'preparation_time_hours',
        'vehicle_count',
        'capacity'
    ].includes(fieldId));

    // Handle bindings
    function handleCheckboxChange(e: Event) {
        const target = e.target as HTMLInputElement;
        listingStore.updateFormData({
            categoryAttributes: {
                ...$listingStore.formData.categoryAttributes,
                [fieldId]: target.checked
            }
        });
    }

    function handleInputChange(e: Event) {
        const target = e.target as HTMLInputElement;
        let val: any = target.value;
        if (isNumber) {
            val = val === '' ? '' : Number(val);
        }
        listingStore.updateFormData({
            categoryAttributes: {
                ...$listingStore.formData.categoryAttributes,
                [fieldId]: val
            }
        });
    }
</script>

{#if isVisible}
    <div class="form-group" class:checkbox-group={isBoolean}>
        {#if isBoolean}
            <label class="checkbox-label">
                <input
                    type="checkbox"
                    checked={!!$listingStore.formData.categoryAttributes[fieldId]}
                    onchange={handleCheckboxChange}
                />
                <span class="label-text">{label}</span>
            </label>
        {:else if isSelect}
            <label for={fieldId} class="field-label">
                {label}
                {#if required}<span class="req">*</span>{/if}
            </label>
            <select
                id={fieldId}
                value={$listingStore.formData.categoryAttributes[fieldId] || ''}
                onchange={handleInputChange}
                required={required}
                class="form-control"
            >
                <option value="">-- {i18n.locale === 'ar' ? 'اختر' : 'Select'} --</option>
                <option value="buffet">{i18n.locale === 'ar' ? 'بوفيه مفتوح' : 'Open Buffet'}</option>
                <option value="plated">{i18n.locale === 'ar' ? 'تقديم أطباق' : 'Plated Service'}</option>
                <option value="both">{i18n.locale === 'ar' ? 'كلاهما' : 'Both'}</option>
            </select>
        {:else if isTextarea}
            <label for={fieldId} class="field-label">
                {label}
                {#if required}<span class="req">*</span>{/if}
            </label>
            <textarea
                id={fieldId}
                value={$listingStore.formData.categoryAttributes[fieldId] ?? ''}
                oninput={handleInputChange}
                required={required}
                class="form-control"
                rows="4"
            ></textarea>
        {:else}
            <label for={fieldId} class="field-label">
                {label}
                {#if required}<span class="req">*</span>{/if}
            </label>
            <input
                id={fieldId}
                type={isNumber ? "number" : "text"}
                value={$listingStore.formData.categoryAttributes[fieldId] ?? ''}
                oninput={handleInputChange}
                required={required}
                class="form-control"
                min="0"
            />
        {/if}
    </div>
{/if}

<style>
    .form-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .checkbox-group {
        flex-direction: row;
        align-items: center;
        padding: 8px 0;
    }
    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        font-size: 0.9rem;
    }
    .field-label {
        font-weight: 500;
        font-size: 0.9rem;
        color: var(--text);
    }
    .req {
        color: var(--danger);
        margin-left: 2px;
    }
    .form-control {
        padding: 10px 12px;
        border: 1px solid var(--border);
        border-radius: 6px;
        background: white;
        color: var(--text);
        font-size: 0.95rem;
        transition: border-color 0.2s;
    }
    .form-control:focus {
        outline: none;
        border-color: var(--primary);
    }
</style>
