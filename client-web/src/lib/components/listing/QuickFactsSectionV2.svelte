<script lang="ts">
    import type { ListingDetail } from "$lib/types/index.js";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import { Users, Car, Building, Sparkles, Compass, Check, LayoutGrid } from "lucide-svelte";

    let { listing } = $props<{ listing: ListingDetail }>();

    const isAr = $derived(getLocale() === 'ar');

    interface Fact {
        id: string;
        icon: any;
        labelAr: string;
        labelEn: string;
        value: string;
    }

    // Config-driven schema definition
    const factConfigs = [
        {
            id: "total_capacity",
            icon: Users,
            labelAr: "إجمالي السعة",
            labelEn: "Total Capacity",
            getValue: (l: ListingDetail) => l.totalCapacity,
            format: (val: any, isAr: boolean) => {
                const num = Number(val);
                if (isNaN(num) || num <= 0) return null;
                return isAr ? `${num} ضيف` : `${num} Guests`;
            }
        },
        {
            id: "men_capacity",
            icon: Users,
            labelAr: "سعة قاعة الرجال",
            labelEn: "Men's Capacity",
            getValue: (l: ListingDetail) => l.culturalAttributes?.men_capacity || l.attributes?.men_capacity,
            format: (val: any, isAr: boolean) => {
                const num = Number(val);
                if (isNaN(num) || num <= 0) return null;
                return isAr ? `${num} شخص` : `${num} Men`;
            }
        },
        {
            id: "women_capacity",
            icon: Users,
            labelAr: "سعة قاعة النساء",
            labelEn: "Women's Capacity",
            getValue: (l: ListingDetail) => l.culturalAttributes?.women_capacity || l.attributes?.women_capacity,
            format: (val: any, isAr: boolean) => {
                const num = Number(val);
                if (isNaN(num) || num <= 0) return null;
                return isAr ? `${num} شخص` : `${num} Women`;
            }
        },
        {
            id: "bridal_suite",
            icon: Sparkles,
            labelAr: "جناح العروس",
            labelEn: "Bridal Suite",
            getValue: (l: ListingDetail) => l.culturalAttributes?.bridal_suite || l.attributes?.bridal_suite,
            format: (val: any, isAr: boolean) => {
                if (val === true || val === "true" || val === "yes") {
                    return isAr ? "متوفر" : "Available";
                }
                return null;
            }
        },
        {
            id: "valet_parking",
            icon: Car,
            labelAr: "خدمة صف السيارات",
            labelEn: "Valet Parking",
            getValue: (l: ListingDetail) => l.culturalAttributes?.valet_parking || l.attributes?.valet_parking,
            format: (val: any, isAr: boolean) => {
                if (val === true || val === "true" || val === "yes") {
                    return isAr ? "متوفر" : "Available";
                }
                return null;
            }
        },
        {
            id: "separate_entrances",
            icon: Compass,
            labelAr: "مداخل منفصلة",
            labelEn: "Separate Entrances",
            getValue: (l: ListingDetail) => l.culturalAttributes?.has_separate_entrances || l.attributes?.has_separate_entrances,
            format: (val: any, isAr: boolean) => {
                if (val === true || val === "true" || val === "yes") {
                    return isAr ? "متوفرة" : "Available";
                }
                return null;
            }
        },
        {
            id: "prayer_room",
            icon: Building,
            labelAr: "مصلى خاص",
            labelEn: "Prayer Room",
            getValue: (l: ListingDetail) => l.culturalAttributes?.prayer_room || l.attributes?.prayer_room,
            format: (val: any, isAr: boolean) => {
                if (val === true || val === "true" || val === "yes") {
                    return isAr ? "متوفر" : "Available";
                }
                return null;
            }
        }
    ];

    // Compute active facts
    const activeFacts = $derived.by(() => {
        const list: Fact[] = [];
        for (const config of factConfigs) {
            const rawVal = config.getValue(listing);
            const formatted = config.format(rawVal, isAr);
            if (formatted) {
                list.push({
                    id: config.id,
                    icon: config.icon,
                    labelAr: config.labelAr,
                    labelEn: config.labelEn,
                    value: formatted
                });
            }
        }
        return list;
    });
</script>

{#if activeFacts.length > 0}
    <!-- GCC Facts V2 Layout Panel -->
    <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm">
        
        <div class="flex items-center gap-3 mb-6 border-b border-slate-100 pb-4">
            <div class="p-2.5 bg-amber-50 text-amber-600 rounded-xl">
                <LayoutGrid size={22} />
            </div>
            <h3 class="font-display text-xl font-bold text-slate-900">
                {isAr ? "معلومات القاعة الأساسية" : "Venue Quick Facts"}
            </h3>
        </div>

        <!-- Multi-column responsive grid -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 gap-4">
            {#each activeFacts as fact (fact.id)}
                {@const Icon = fact.icon}
                <div class="flex items-start gap-3 p-4 rounded-xl bg-slate-50/60 border border-slate-100/80 hover:bg-slate-50 transition-colors">
                    <div class="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center shrink-0">
                        <Icon size={16} />
                    </div>
                    <div class="space-y-0.5">
                        <p class="text-[10px] sm:text-xs font-bold text-slate-400 uppercase tracking-wide">
                            {isAr ? fact.labelAr : fact.labelEn}
                        </p>
                        <p class="text-sm font-extrabold text-slate-800">
                            {fact.value}
                        </p>
                    </div>
                </div>
            {/each}
        </div>

    </section>
{/if}
