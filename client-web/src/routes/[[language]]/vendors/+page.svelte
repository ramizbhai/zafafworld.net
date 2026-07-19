<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';
  import { vendorService } from '$lib/services/api/vendor.service.js';
import Badge from "$lib/components/ui/Badge.svelte";
import StarRating from "$lib/components/ui/StarRating.svelte";
import Button from "$lib/components/ui/Button.svelte";

  let { data } = $props();
  let vendors = $derived(data.vendors || []);
</script>

<svelte:head>
  <title>{m.nav_vendors()} - {m.meta_siteName()}</title>
</svelte:head>

<!-- Header -->
<div class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
  <div class="container-page py-12">
    <span class="divider-gold"></span>
    <h1 class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-4 mb-2">
      {m.nav_vendors()}
    </h1>
    <p class="text-[var(--color-muted)]">
      {m.auto_meet_the_top_verifie()}
    </p>
  </div>
</div>

<div class="container-page py-12">
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
    {#if vendors.length === 0}
      <div class="col-span-full py-24 text-center text-[var(--color-muted)] border border-dashed border-[var(--color-border)] rounded-2xl bg-[var(--color-surface-alt)]">
        <span class="text-5xl mb-4 block">🏢</span>
        <p class="font-display text-2xl font-semibold text-[var(--color-secondary)] mb-2">
          {m.auto_no_service_providers()}
        </p>
        <p class="text-sm mt-1">{m.auto_please_check_back_la()}</p>
      </div>
    {:else}
      {#each vendors as vendor (vendor.id)}
        <article class="bg-white rounded-2xl border border-[var(--color-border)] overflow-hidden card-hover">
          <!-- Cover -->
          <div class="h-40 bg-[var(--color-surface-alt)] overflow-hidden">
            {#if vendor.preview}
              <img src={resolveMediaUrl(getOptimizedImage(vendor.preview, 'card'))} alt="" class="w-full h-full object-cover" loading="lazy" />
            {:else}
              <div class="w-full h-full flex items-center justify-center text-4xl">🏛️</div>
            {/if}
          </div>

          <div class="p-6">
            <!-- Avatar + name -->
            <div class="flex items-start gap-4 mb-4">
              <div class="w-14 h-14 rounded-xl bg-[var(--color-secondary)] flex items-center justify-center font-display text-xl font-bold text-[var(--color-primary)] flex-shrink-0 -mt-10 border-2 border-white shadow-[var(--shadow-md)]">
                {(getLocalizedField(vendor, 'name', getLocale())).charAt(0)}
              </div>
              <div class="min-w-0 mt-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <h2 class="font-semibold text-[var(--color-secondary)] truncate">
                    {getLocalizedField(vendor, 'name', getLocale())}
                  </h2>
                  {#if vendor.verified}
                    <Badge variant="success" size="sm">✓</Badge>
                  {/if}
                </div>
                <p class="text-xs text-[var(--color-muted)] mt-0.5">{vendor.city}</p>
              </div>
            </div>

            <!-- Stats -->
            <div class="flex items-center justify-between mb-4 text-sm">
              <StarRating rating={vendor.rating} showValue size="sm" />
              <span class="text-[var(--color-muted)]">
                {vendor.venueCount} {m.auto_venues()}
              </span>
            </div>

            <Button
              href="/venues?vendor={vendor.id}"
              variant="outline"
              fullWidth
              size="sm"
            >
              {m.auto_view_venues()}
            </Button>
          </div>
        </article>
      {/each}
    {/if}
  </div>
</div>
