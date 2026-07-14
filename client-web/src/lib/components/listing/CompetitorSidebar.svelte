<script lang="ts">
  import { onMount } from "svelte";
  import ListingCard from "$lib/components/shared/ListingCard.svelte";
  import InquiryForm from "$lib/components/shared/InquiryForm.svelte";
  import type { ListingDetail, Listing } from "$lib/types/index.js";
  import { ShieldCheck, PhoneCall, Globe } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";

  import { listingService } from "$lib/services/api/listing.service.js";

  let { listing } = $props<{ listing: ListingDetail }>();

  const isAr = $derived(getLocale() === "ar");

  // Strict Tier Validation Logic
  const tierId = $derived(listing.subscriptionBadge?.tierId || "free");
  const isDiamond = $derived(tierId === "diamond");
  const isVip = $derived(tierId === "vip");
  const isGold = $derived(tierId === "gold");
  const isFree = $derived(tierId === "free");

  // Diamond & VIP: Protect the lead by hiding competitors
  const hideCompetitors = $derived(isDiamond || isVip);

  // Diamond & VIP: Reveal direct contact info
  const showDirectContact = $derived(isDiamond || isVip);

  let relatedListings = $state<Listing[]>([]);
  let loading = $state(true);

  onMount(async () => {
    // If competitors are hidden, do not fetch them
    if (hideCompetitors) {
      loading = false;
      return;
    }

    try {
      const data = await listingService.getAll({
        category: listing.category,
        limit: 4,
      });

      relatedListings = (data.listings || [])
        .filter((l: Listing) => l.id !== listing.id)
        .slice(0, 3);
    } catch (e) {
      console.error("Failed to fetch related listings", e);
    } finally {
      loading = false;
    }
  });
</script>

<aside class="sticky top-24 space-y-6">
  <!-- Exclusive Contact Block (Only for Diamond & VIP) -->
  {#if showDirectContact && (listing.vendor?.phone || listing.vendor?.website)}
    <div
      class="bg-white rounded-2xl border border-gray-200 shadow-sm p-5 space-y-4"
    >
      <h3
        class="text-sm font-bold text-slate-800 uppercase tracking-wider mb-2"
      >
        {isAr ? "بيانات الاتصال المباشر" : "Direct Contact Info"}
      </h3>

      {#if listing.vendor.phone}
        <div class="flex items-center gap-3 text-slate-700">
          <div class="p-2 bg-blue-50 text-blue-600 rounded-full">
            <PhoneCall size={18} />
          </div>
          <a
            href="tel:{listing.vendor.phone}"
            class="font-medium hover:text-blue-600 transition-colors"
            dir="ltr"
          >
            {listing.vendor.phone}
          </a>
        </div>
      {/if}

      {#if listing.vendor.website}
        <div class="flex items-center gap-3 text-slate-700">
          <div class="p-2 bg-purple-50 text-purple-600 rounded-full">
            <Globe size={18} />
          </div>
          <a
            href={listing.vendor.website}
            target="_blank"
            rel="noopener noreferrer"
            class="font-medium hover:text-purple-600 transition-colors truncate"
          >
            {isAr ? "زيارة الموقع الإلكتروني" : "Visit Website"}
          </a>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Inquiry Form -->
  <div
    class="bg-white rounded-2xl border border-gray-100 shadow-xl overflow-hidden relative"
  >
    <!-- Diamond Exclusive Label -->
    {#if isDiamond}
      <div
        class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-cyan-400 to-blue-600"
      ></div>
    {:else if isVip}
      <div
        class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-purple-500 to-indigo-600"
      ></div>
    {:else if isGold}
      <div
        class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-amber-400 to-orange-500"
      ></div>
    {/if}

    <div class="p-6">
      <h3 class="text-xl font-bold text-slate-800 mb-2">
        {isAr ? "طلب تسعير ومعلومات" : "Request Pricing & Availability"}
      </h3>
      <p class="text-sm text-slate-500 mb-6">
        {isAr
          ? "تواصل مباشرة مع منسق المناسبات."
          : `Connect directly with the event coordinator for ${listing.titleEn || listing.title || ""}.`}
      </p>

      <InquiryForm listingId={listing.id} />
    </div>

    <!-- Trust Badge -->
    <div
      class="bg-slate-50 px-6 py-4 flex items-center gap-3 border-t border-gray-100"
    >
      <ShieldCheck class="text-emerald-500 shrink-0" size={24} />
      <span class="text-xs text-slate-500 font-medium">
        {isAr
          ? "استفسار آمن. يتم مشاركة بياناتك مع هذا المزود فقط."
          : "Safe Inquiry. Your details are protected and only shared with this vendor."}
      </span>
    </div>
  </div>

  <!-- Exclusive Partnership Badges (Replaces Competitors) -->
  {#if isDiamond}
    <div
      class="p-6 bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-lg border border-slate-700 text-center"
    >
      <span class="inline-block p-3 bg-white/10 rounded-full mb-3 text-2xl">
        💎
      </span>
      <h4 class="text-white font-bold mb-2">
        {isAr ? "شريك دايموند" : "Diamond Partner"}
      </h4>
      <p class="text-sm text-slate-300">
        {isAr
          ? "هذا المزود شريك دايموند حصري، يضمن أعلى مستوى من الخدمة وتجارب المناسبات المميزة."
          : "This vendor is an exclusive Diamond Partner, ensuring the highest level of service and premium event experiences."}
      </p>
    </div>
  {:else if isVip}
    <div
      class="p-6 bg-gradient-to-br from-purple-900 to-indigo-900 rounded-2xl shadow-lg border border-purple-800 text-center"
    >
      <span class="inline-block p-3 bg-white/10 rounded-full mb-3 text-2xl">
        👑
      </span>
      <h4 class="text-white font-bold mb-2">
        {isAr ? "شريك مميز" : "VIP Partner"}
      </h4>
      <p class="text-sm text-purple-200">
        {isAr
          ? "مزود خدمة موثوق يقدم جودة استثنائية واحترافية عالية."
          : "A trusted service provider delivering exceptional quality and high professionalism."}
      </p>
    </div>
  {:else}
    <!-- Competitor Section for Gold/Free -->
    <div class="space-y-4">
      <h3 class="text-lg font-bold text-slate-800">
        {isAr ? "خيارات مشابهة" : "Similar Venues"}
      </h3>
      {#if loading}
        <div class="space-y-4">
          <div class="h-[300px] bg-slate-100 rounded-2xl animate-pulse"></div>
          <div class="h-[300px] bg-slate-100 rounded-2xl animate-pulse"></div>
        </div>
      {:else if relatedListings.length > 0}
        <div class="flex flex-col gap-6">
          {#each relatedListings as related (related.id)}
            <ListingCard listing={related} layout="grid" />
          {/each}
        </div>
      {:else}
        <p class="text-sm text-slate-500">
          {isAr ? "لا توجد خيارات مشابهة." : "No similar venues found."}
        </p>
      {/if}
    </div>
  {/if}
</aside>
