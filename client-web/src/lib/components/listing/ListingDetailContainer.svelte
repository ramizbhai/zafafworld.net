<script lang="ts">
    import ImmersiveGallery from "./ImmersiveGallery.svelte";
    import AdvancedChecklist from "./AdvancedChecklist.svelte";
    import CompetitorSidebar from "./CompetitorSidebar.svelte";
    import UnifiedDescriptionBuilder from "./UnifiedDescriptionBuilder.svelte";
    import DynamicListingAttributes from "./DynamicListingAttributes.svelte";
    import CoordinatorContactCard from "./CoordinatorContactCard.svelte";
    import ListingFeaturesGrid from "./ListingFeaturesGrid.svelte";
    import ListingDetailSkeleton from "./ListingDetailSkeleton.svelte";
    import { MapPin, Users, Building, ShieldCheck } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import * as m from "$lib/paraglide/messages.js";

    let { data = {} } = $props<{ data: any }>();

    const listing = $derived(data?.listing);
    const isLoading = $derived(data?.loading || (!data?.listing && data?.initialLoading !== false));
    const isAr = $derived(getLocale() === 'ar');

    // Extractors
    const attributes = $derived(listing?.attributes || {});
    const featuresSelection = $derived(listing?.featuresSelection || {});
    const images = $derived(listing?.images || []);
    const coordinator = $derived(listing?.coordinator);
    const vendor = $derived(listing?.vendor);

    const title = $derived(
        isAr
            ? (listing?.titleAr || listing?.title || "")
            : (listing?.titleEn || listing?.title || "")
    );

    const descriptionBlocksAr = $derived(listing?.descriptionAr || "");
    const descriptionBlocksEn = $derived(listing?.descriptionEn || "");

    const city = $derived(isAr ? (listing?.cityAr || listing?.cityEn || "") : (listing?.cityEn || listing?.cityAr || ""));
    
    // Badge details
    const tierId = $derived(listing?.subscriptionBadge?.tierId || "free");
    const badgeLabel = $derived(
        isAr
            ? (listing?.subscriptionBadge?.ar || listing?.subscriptionBadge?.en || "مميز") 
            : (listing?.subscriptionBadge?.en || "Standard")
    );

    // Gender section badge label helper
    const genderSection = $derived(listing?.genderSection);
    const genderSectionLabel = $derived.by(() => {
        if (!genderSection) return null;
        switch (genderSection) {
            case 'dual_parallel':
                return isAr ? '🏰 قسمين منفصلين (رجال ونساء)' : '🏰 Dual Parallel (Separated)';
            case 'women_only':
                return isAr ? '👑 قاعة مخصصة للنساء فقط' : '👑 Women Only';
            case 'family':
            case 'mixed':
                return isAr ? '👨‍👩‍👧‍👦 عائلي / مختلط' : '👨‍👩‍👧‍👦 Family / Mixed';
            case 'men_only':
                return isAr ? '🕺 مخصص للرجال فقط' : '🕺 Men Only';
            default:
                return null;
        }
    });

    const hasLocation = $derived(!!(listing?.googleMapsUrl || city || listing?.latitude));

    const showDescriptionSection = $derived.by(() => {
        const descAr = descriptionBlocksAr;
        const descEn = descriptionBlocksEn;
        if (!descAr && !descEn) return false;
        
        const hasContent = (val: string) => {
            if (!val || val.trim() === "" || val.trim() === "[]") return false;
            try {
                const parsed = JSON.parse(val);
                if (Array.isArray(parsed)) {
                    return parsed.some(b => 
                        b.type === "divider" || 
                        (b.contentAr && b.contentAr.trim() !== "") ||
                        (b.contentEn && b.contentEn.trim() !== "") ||
                        (b.content && b.content.trim() !== "") || 
                        (b.url && b.url.trim() !== "")
                    );
                }
                return true;
            } catch {
                return true;
            }
        };
        return hasContent(descAr) || hasContent(descEn);
    });
</script>

{#if isLoading}
    <ListingDetailSkeleton />
{:else if !listing}
    <div class="min-h-screen flex items-center justify-center bg-slate-50">
        <div class="text-center p-8 bg-white rounded-2xl border border-slate-200 shadow-sm">
            <h2 class="text-2xl font-bold text-slate-800">{isAr ? 'الإعلان غير موجود' : 'Listing not found'}</h2>
            <a
                href="/"
                class="text-amber-600 hover:text-amber-700 font-semibold underline mt-4 inline-block"
            >
                {isAr ? 'العودة للبحث' : 'Return to search'}
            </a>
        </div>
    </div>
{:else}
    <article class="bg-slate-50/50 min-h-screen pb-24 pt-6">
        <!-- 1. Full-Screen Immersive Gallery (The Top Banner) -->
        <ImmersiveGallery {images} {title} />

        <!-- Main Container -->
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-8 sm:mt-12">
            
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-12">
                <!-- Main Content Column (Left on Desktop) -->
                <div class="lg:col-span-8 space-y-8">
                    <!-- Top Title Area & GCC Badges -->
                    <div class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm">
                        <div class="flex flex-wrap items-center gap-2.5 mb-4">
                            <!-- Subscription Tier Badge -->
                            {#if tierId !== "free"}
                                <span class="px-3 py-1 text-xs font-bold uppercase tracking-wide bg-gradient-to-r from-amber-500 to-amber-600 text-slate-950 rounded-full shadow-xs">
                                    {badgeLabel}
                                </span>
                            {/if}

                            <!-- GCC Gender Section Badge -->
                            {#if genderSectionLabel}
                                <span class="px-3.5 py-1 text-xs font-bold bg-slate-900 text-amber-300 rounded-full border border-slate-800 shadow-xs">
                                    {genderSectionLabel}
                                </span>
                            {/if}

                            <!-- City Badge -->
                            {#if city}
                                <span class="px-3 py-1 text-xs font-medium bg-slate-100 text-slate-700 rounded-full flex items-center gap-1">
                                    <MapPin size={12} class="text-amber-500" /> {city}
                                </span>
                            {/if}
                        </div>

                        <h1 class="text-3xl sm:text-4xl font-extrabold text-slate-900 leading-tight mb-2">{title}</h1>
                    </div>

                    <!-- Category-Specific Dynamic Specifications -->
                    <DynamicListingAttributes {attributes} category={listing.category} />

                    <!-- Detailed Description -->
                    {#if showDescriptionSection}
                        <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm">
                            <h3 class="font-display text-xl font-bold text-slate-900 mb-6 border-b border-slate-100 pb-4">
                                {isAr ? 'تفاصيل الخدمة الوصفية' : 'Service Description'}
                            </h3>
                            <UnifiedDescriptionBuilder 
                                descriptionAr={descriptionBlocksAr} 
                                descriptionEn={descriptionBlocksEn} 
                                readonly={true}
                                locale={getLocale() as 'ar' | 'en'}
                            />
                        </section>
                    {/if}

                    <!-- Additional Features & Cultural Amenities -->
                    <ListingFeaturesGrid {featuresSelection} culturalAttributes={listing.culturalAttributes || attributes} />

                    <!-- Standard Legacy Checklist (if needed) -->
                    {#if attributes && Object.keys(attributes).length > 0}
                        <section>
                            <AdvancedChecklist {attributes} />
                        </section>
                    {/if}

                    <!-- Coordinator & Direct Contact Card -->
                    <CoordinatorContactCard {coordinator} {vendor} />

                    <!-- Location Map Section -->
                    {#if hasLocation}
                        <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm">
                            <h3 class="font-display text-xl font-bold text-slate-900 mb-6 border-b border-slate-100 pb-4">
                                {m.listing_location_map_title()}
                            </h3>
                            <div class="flex items-start gap-4">
                                <div class="p-3 bg-amber-50 text-amber-600 rounded-2xl shrink-0">
                                    <MapPin size={24} />
                                </div>
                                <div class="space-y-2">
                                    <p class="text-lg font-bold text-slate-900">{city || m.listing_location_unavailable()}</p>
                                    {#if listing.googleMapsUrl}
                                        <a 
                                            href={listing.googleMapsUrl} 
                                            target="_blank" 
                                            rel="noopener noreferrer" 
                                            class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-semibold text-xs shadow-sm transition-all mt-2"
                                        >
                                            <MapPin size={14} class="text-amber-400" />
                                            <span>{m.listing_view_on_google_maps()}</span>
                                        </a>
                                    {/if}
                                </div>
                            </div>
                        </section>
                    {/if}
                </div>

                <!-- Sidebar Column (Right on Desktop) -->
                <div class="lg:col-span-4 space-y-6">
                    <CompetitorSidebar {listing} />
                </div>
            </div>
        </div>
    </article>
{/if}
