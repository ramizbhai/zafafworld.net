<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import { env } from '$env/dynamic/public';
</script>

<section class="py-0 overflow-hidden" aria-label="Call to action">
  <div class="grid grid-cols-1 lg:grid-cols-2">
    <!-- Customer CTA -->
    <div class="relative flex flex-col items-center justify-center text-center px-8 py-20 overflow-hidden bg-[var(--color-primary)]">
      <!-- Background decoration -->
      <div class="absolute inset-0 opacity-10" aria-hidden="true">
        <div class="absolute -top-20 -start-20 w-64 h-64 rounded-full bg-white"></div>
        <div class="absolute -bottom-20 -end-20 w-80 h-80 rounded-full bg-[var(--color-secondary)]"></div>
      </div>

      <div class="relative z-10">
        <div class="text-5xl mb-6" aria-hidden="true">💍</div>
        <h2 class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mb-4">
          {m.home_cta_title()}
        </h2>
        <p class="text-[var(--color-secondary)]/70 text-base mb-8 max-w-sm mx-auto leading-relaxed">
          {m.home_cta_subtitle()}
        </p>
        <Button href="/venues" variant="secondary" size="lg">
          {m.home_cta_button()}
        </Button>
      </div>
    </div>

    <!-- Vendor CTA -->
    <div class="relative flex flex-col items-center justify-center text-center px-8 py-20 overflow-hidden bg-[var(--color-secondary)]">
      <div class="absolute inset-0 opacity-10" aria-hidden="true">
        <div class="absolute -top-20 -end-20 w-64 h-64 rounded-full bg-[var(--color-primary)]"></div>
        <div class="absolute -bottom-20 -start-20 w-80 h-80 rounded-full bg-white"></div>
      </div>

      <div class="relative z-10">
        <div class="text-5xl mb-6" aria-hidden="true">🏛️</div>
        <h2 class="font-display text-3xl sm:text-4xl font-bold text-white mb-4">
          {m.home_cta_vendorTitle()}
        </h2>
        <p class="text-white/60 text-base mb-8 max-w-sm mx-auto leading-relaxed">
          {m.home_cta_vendorSubtitle()}
        </p>
        <Button href="{env.PUBLIC_VENDOR_URL || 'http://localhost:5174'}/register" variant="primary" size="lg">
          {m.home_cta_vendorButton()}
        </Button>
      </div>
    </div>
  </div>
</section>
