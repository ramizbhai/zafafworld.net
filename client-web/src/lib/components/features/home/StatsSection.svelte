<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import {
    getLocalizedField,
    formatCurrency,
    formatNumber,
    formatDate,
  } from "$lib/utils/localize.js";
  import { Landmark, CalendarCheck, MapPin, Star } from "lucide-svelte";

  let { serverStats = null }: { serverStats?: any } = $props();

  const stats = $derived([
    {
      value: serverStats?.venues || 0,
      suffix: "+",
      label: m.home_stats_venues(),
      Icon: Landmark,
    },
    {
      value: serverStats?.bookings || 0,
      suffix: "+",
      label: m.home_stats_bookings(),
      Icon: CalendarCheck,
    },
    {
      value: serverStats?.cities || 0,
      suffix: "",
      label: m.home_stats_cities(),
      Icon: MapPin,
    },
    {
      value: serverStats?.satisfaction ?? null,
      suffix: "%",
      label: m.home_stats_satisfaction(),
      Icon: Star,
    },
  ]);
</script>

<section
  class="py-20 bg-zw-secondary relative overflow-hidden"
  aria-label="Statistics"
>
  <!-- Subtle background glow -->
  <div
    class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-zw-primary/5 via-transparent to-transparent"
  ></div>

  <div class="container-page relative z-10">
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-y-12 gap-x-8">
      {#each stats as stat, i}
        <div class="relative text-center group flex flex-col items-center">
          <div
            class="mb-5 p-4 rounded-full bg-zw-primary/5 text-zw-primary border border-zw-primary/20 transform group-hover:-translate-y-2 group-hover:bg-zw-primary group-hover:text-zw-secondary group-hover:shadow-gold transition-all duration-500 ease-out"
          >
            <stat.Icon size={32} strokeWidth={1.5} />
          </div>

          <div
            class="font-display text-4xl sm:text-5xl font-bold text-gradient-gold mb-2 drop-shadow-sm transition-transform duration-500 group-hover:scale-105"
          >
            {#if stat.value === null}
              <span class="text-xl sm:text-2xl opacity-70 tracking-normal font-medium whitespace-nowrap">No reviews</span>
            {:else}
              {formatNumber(stat.value)}{stat.suffix}
            {/if}
          </div>

          <div
            class="text-xs sm:text-sm uppercase tracking-widest text-white/60 font-medium mt-1"
          >
            {stat.label}
          </div>

          {#if i < stats.length - 1}
            <!-- Desktop Divider -->
            <div
              class="hidden lg:block absolute inset-y-4 end-[-1rem] w-px bg-gradient-to-b from-transparent via-zw-primary/20 to-transparent"
              aria-hidden="true"
            ></div>
          {/if}

          <!-- Mobile Divider (only after 1st and 3rd item) -->
          {#if i % 2 === 0}
            <div
              class="block lg:hidden absolute inset-y-4 end-[-1rem] w-px bg-gradient-to-b from-transparent via-zw-primary/20 to-transparent"
              aria-hidden="true"
            ></div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</section>
