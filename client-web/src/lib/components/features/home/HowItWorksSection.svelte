<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  const steps = $derived([
    {
      number: '01',
      title:       m.home_steps_search_title(),
      description: m.home_steps_search_description(),
      icon: `<svg viewBox="0 0 48 48" fill="none" class="w-10 h-10">
        <path d="M21 38C30.3888 38 38 30.3888 38 21C38 11.6112 30.3888 4 21 4C11.6112 4 4 11.6112 4 21C4 30.3888 11.6112 38 21 38Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>
        <path d="M26.6568 14.3431C28.2828 15.9691 29.2828 18.2691 29.2828 20.7828" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M33.2218 33.2218L44 44" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>`
    },
    {
      number: '02',
      title:       m.home_steps_compare_title(),
      description: m.home_steps_compare_description(),
      icon: `<svg viewBox="0 0 48 48" fill="none" class="w-10 h-10">
        <path d="M24 44C35.0457 44 44 35.0457 44 24C44 12.9543 35.0457 4 24 4C12.9543 4 4 12.9543 4 24C4 35.0457 12.9543 44 24 44Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>
        <path d="M24 16V32" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M16 24L24 32L32 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>`
    },
    {
      number: '03',
      title:       m.home_steps_book_title(),
      description: m.home_steps_book_description(),
      icon: `<svg viewBox="0 0 48 48" fill="none" class="w-10 h-10">
        <path d="M40 20V36a4 4 0 01-4 4H12a4 4 0 01-4-4V20" stroke="currentColor" stroke-width="3"/>
        <path d="M8 20h32M16 8v8M32 8v8" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
        <path d="M18 30l4 4 8-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>`,
    },
  ]);
</script>

<section class="py-24 bg-[var(--color-surface)]" aria-labelledby="how-it-works-title">
  <div class="container-page">
    <!-- Section Header -->
    <div class="text-center mb-16">
      <span class="divider-gold mx-auto"></span>
      <h2 id="how-it-works-title" class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-6 mb-4">
        {m.home_howItWorks()}
      </h2>
      <p class="text-[var(--color-muted)] text-lg max-w-xl mx-auto">
        {m.home_howItWorksSubtitle()}
      </p>
    </div>

    <!-- Steps grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-8 relative">
      <!-- Connector line -->
      <div
        class="hidden md:block absolute top-16 inset-x-0 h-px bg-gradient-to-r from-transparent via-[var(--color-border)] to-transparent z-0"
        aria-hidden="true"
      ></div>

      {#each steps as step, i}
        <div class="relative z-10 flex flex-col items-center text-center group">
          <!-- Step number bubble -->
          <div class="
            relative w-20 h-20 rounded-2xl bg-white border-2 border-[var(--color-border)]
            flex items-center justify-center mb-6 shadow-[var(--shadow-md)]
            group-hover:border-[var(--color-primary)] group-hover:shadow-[var(--shadow-gold)]
            transition-all duration-300
          ">
            <div class="text-[var(--color-primary)]">
              {@html step.icon}
            </div>
            <div class="
              absolute -top-3 -end-3 w-7 h-7 rounded-full
              bg-[var(--color-primary)] text-[var(--color-secondary)]
              flex items-center justify-center text-xs font-bold
              shadow-[var(--shadow-sm)]
            ">
              {i + 1}
            </div>
          </div>

          <h3 class="font-display text-xl font-semibold text-[var(--color-secondary)] mb-3">
            {step.title}
          </h3>
          <p class="text-[var(--color-muted)] text-sm leading-relaxed max-w-xs">
            {step.description}
          </p>
        </div>
      {/each}
    </div>
  </div>
</section>
