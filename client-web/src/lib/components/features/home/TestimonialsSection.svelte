<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import StarRating from '$lib/components/ui/StarRating.svelte';

  let { testimonials = [] }: { testimonials?: any[] } = $props();

  let activeIndex = $state(0);

  function prev() { 
    if (testimonials.length > 0) activeIndex = (activeIndex - 1 + testimonials.length) % testimonials.length; 
  }
  function next() { 
    if (testimonials.length > 0) activeIndex = (activeIndex + 1) % testimonials.length; 
  }
</script>

<section class="py-24 bg-[var(--color-surface-alt)]" aria-labelledby="testimonials-title">
  {#if testimonials && testimonials.length > 0}
    <div class="container-page">
      <!-- Header -->
      <div class="text-center mb-16">
        <span class="divider-gold mx-auto"></span>
        <h2 id="testimonials-title" class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-6 mb-4">
          {m.home_testimonials()}
        </h2>
        <p class="text-[var(--color-muted)] text-lg">{m.home_testimonialsSubtitle()}</p>
      </div>

      <!-- Desktop: all 3 cards -->
      <div class="hidden md:grid md:grid-cols-3 gap-8">
        {#each testimonials as t, i}
          <article
            class="bg-white rounded-2xl p-8 border border-[var(--color-border)] shadow-[var(--shadow-sm)] hover:shadow-[var(--shadow-lg)] transition-shadow duration-300 flex flex-col gap-5"
            aria-label={getLocalizedField(t, 'name', getLocale())}
          >
            <!-- Quote icon -->
            <div class="text-[var(--color-primary)] opacity-30 font-display text-6xl leading-none select-none" aria-hidden="true">
              "
            </div>

            <StarRating rating={t.rating} size="sm" />

            <p class="text-[var(--color-text)] text-sm leading-relaxed flex-1">
              {getLocalizedField(t, 'text', getLocale())}
            </p>

            <div class="flex items-center gap-4 pt-4 border-t border-[var(--color-border)]">
              <img
                src={t.image}
                alt={getLocalizedField(t, 'name', getLocale())}
                class="w-12 h-12 rounded-full object-cover border-2 border-[var(--color-primary-light)]"
                loading="lazy"
              />
              <div>
                <p class="font-semibold text-[var(--color-secondary)] text-sm">
                  {getLocalizedField(t, 'name', getLocale())}
                </p>
                <p class="text-xs text-[var(--color-muted)]">
                  {getLocalizedField(t.city, 'name', getLocale())}
                </p>
              </div>
            </div>
          </article>
        {/each}
      </div>

      <!-- Mobile: carousel -->
      <div class="md:hidden">
        {#if testimonials[activeIndex]}
        <article class="bg-white rounded-2xl p-8 border border-[var(--color-border)] shadow-[var(--shadow-md)] flex flex-col gap-5">
          <div class="text-[var(--color-primary)] opacity-30 font-display text-6xl leading-none" aria-hidden="true">
            "
          </div>
          <StarRating rating={testimonials[activeIndex].rating} size="sm" />
          <p class="text-[var(--color-text)] text-sm leading-relaxed">
            {getLocalizedField(testimonials[activeIndex], 'text', getLocale())}
          </p>
          <div class="flex items-center gap-4 pt-4 border-t border-[var(--color-border)]">
            <img
              src={testimonials[activeIndex].image}
              alt={getLocalizedField(testimonials[activeIndex], 'name', getLocale())}
              class="w-12 h-12 rounded-full object-cover"
            />
            <div>
              <p class="font-semibold text-[var(--color-secondary)] text-sm">
                {getLocalizedField(testimonials[activeIndex], 'name', getLocale())}
              </p>
              <p class="text-xs text-[var(--color-muted)]">
                {getLocalizedField(testimonials[activeIndex], 'city', getLocale())}
              </p>
            </div>
          </div>
        </article>
        {/if}

        <!-- Carousel controls -->
        <div class="flex items-center justify-center gap-4 mt-6">
          <button
            onclick={prev}
            class="w-10 h-10 rounded-full border border-[var(--color-border)] flex items-center justify-center hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors"
            aria-label={m.a11y_previousImage()}
          >
            <svg viewBox="0 0 20 20" class="w-4 h-4 ltr:rotate-180" fill="currentColor">
              <path fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z" clip-rule="evenodd"/>
            </svg>
          </button>

          <div class="flex gap-2" aria-label="Slide indicators">
            {#each testimonials as _, i}
              <button
                onclick={() => activeIndex = i}
                class="w-2 h-2 rounded-full transition-all duration-200 {i === activeIndex ? 'bg-[var(--color-primary)] w-6' : 'bg-[var(--color-border)]'}"
                aria-label="Slide {i + 1}"
                aria-current={i === activeIndex}
              ></button>
            {/each}
          </div>

          <button
            onclick={next}
            class="w-10 h-10 rounded-full border border-[var(--color-border)] flex items-center justify-center hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors"
            aria-label={m.a11y_nextImage()}
          >
            <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
              <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>
