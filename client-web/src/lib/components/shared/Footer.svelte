<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';
  import { i18n } from '$lib/i18n.js';
  import { env } from '$env/dynamic/public';

  const currentYear = new Date().getFullYear();

  const quickLinks = $derived([
    { href: i18n.resolveRoute('/', getLocale()),        label: m.nav_home() },
    { href: i18n.resolveRoute('/venues', getLocale()),  label: m.nav_venues() },
    { href: i18n.resolveRoute('/offers', getLocale()),  label: getLocale() === 'ar' ? 'العروض الحصرية' : 'Exclusive Offers' },
    { href: i18n.resolveRoute('/discover', getLocale()),    label: m.nav_blog() },
    { href: i18n.resolveRoute('/about', getLocale()),   label: m.nav_about() },
  ]);

  const supportLinks = $derived([
    { href: i18n.resolveRoute('/contact', getLocale()),  label: m.footer_contactUs() },
    { href: i18n.resolveRoute('/faq', getLocale()),      label: m.footer_faq() },
    { href: i18n.resolveRoute('/help', getLocale()),     label: m.footer_helpCenter() },
  ]);

  const legalLinks = $derived([
    { href: i18n.resolveRoute('/terms', getLocale()),   label: m.footer_terms() },
    { href: i18n.resolveRoute('/privacy', getLocale()), label: m.footer_privacy() },
    { href: i18n.resolveRoute('/cookies', getLocale()), label: getLocale() === 'ar' ? 'سياسة ملفات تعريف الارتباط' : 'Cookie Policy' },
    { href: i18n.resolveRoute('/cancellation', getLocale()), label: getLocale() === 'ar' ? 'سياسة الإلغاء والاسترجاع' : 'Cancellation & Refund Policy' },
  ]);

  const socialLinks = [
    { href: '#', label: 'Instagram', icon: `<svg viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5"><path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z"/></svg>` },
    { href: '#', label: 'Twitter/X', icon: `<svg viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>` },
    { href: '#', label: 'Snapchat', icon: `<svg viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5"><path d="M12.166.5C9.003.5 6.393 1.538 4.72 3.432c-1.406 1.607-2.099 3.836-2.099 6.634v.637c-.404.194-.783.288-1.127.288-.344 0-.623-.065-.835-.194l-.257-.15-.288.022c-.021 0-.043.001-.064.001-.646 0-1.047.378-1.047.994 0 .518.322.959.968 1.311.066.036.178.087.328.152l.001.001c.26.111.647.276 1.05.578-.208.663-.636 1.137-1.208 1.36l-.003.001C.497 15.185 0 15.558 0 16.209c0 .538.384.966 1.021 1.155.258.078.614.137 1.061.175.036.003.073.006.112.009.132.011.26.042.352.128.132.123.174.362.257.697l.001.005.001.005c.02.089.041.181.065.275l.007.029.008.029c.113.402.39.661.737.661.177 0 .377-.059.591-.175.305-.163.651-.245 1.029-.245.394 0 .814.098 1.246.292.645.29 1.295.438 1.932.438.634 0 1.291-.156 1.951-.463.384-.176.756-.266 1.104-.266.374 0 .723.095 1.04.282.218.127.419.191.598.191.359 0 .629-.265.738-.666.025-.094.046-.186.066-.276l.002-.008c.082-.333.124-.573.256-.695.092-.086.22-.117.352-.128.038-.003.076-.006.112-.009.448-.038.803-.097 1.062-.175.636-.189 1.02-.617 1.02-1.155 0-.651-.497-1.024-.941-1.151-.567-.222-.996-.697-1.205-1.36.403-.302.79-.467 1.05-.578l.001-.001c.15-.065.262-.116.328-.152.646-.352.968-.793.968-1.311 0-.616-.401-.994-1.047-.994a.94.94 0 01-.065-.001l-.257.022-.25.148c-.213.129-.492.194-.835.194-.345 0-.723-.094-1.128-.288v-.637c0-2.798-.692-5.027-2.099-6.634C17.773 1.538 15.163.5 12 .5h.166z"/></svg>` },
    { href: '#', label: 'TikTok', icon: `<svg viewBox="0 0 24 24" fill="currentColor" class="w-5 h-5"><path d="M19.59 6.69a4.83 4.83 0 01-3.77-4.25V2h-3.45v13.67a2.89 2.89 0 01-2.88 2.5 2.89 2.89 0 01-2.89-2.89 2.89 2.89 0 012.89-2.89c.28 0 .54.04.79.1V9.01a6.33 6.33 0 00-.79-.05 6.34 6.34 0 00-6.34 6.34 6.34 6.34 0 006.34 6.34 6.34 6.34 0 006.33-6.34V8.99a8.18 8.18 0 004.78 1.53V7.07a4.85 4.85 0 01-1.01-.38z"/></svg>` },
  ];

  let email = $state('');
  let subscribed = $state(false);

  function handleSubscribe(e: SubmitEvent) {
    e.preventDefault();
    if (email) {
      subscribed = true;
      email = '';
    }
  }
</script>

<footer class="bg-[var(--color-secondary)] text-[var(--color-text-inverse)]">
  <!-- Main Footer -->
  <div class="container-page py-16">
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-12">

      <!-- Brand Column -->
      <div class="lg:col-span-1">
        <a href={i18n.resolveRoute('/', getLocale())} class="flex items-center gap-3 group mb-6">
          <div class="w-10 h-10 rounded-xl bg-[var(--color-primary)] flex items-center justify-center shadow-[var(--shadow-gold)] select-none">
            <span class="font-display text-xl font-black text-[var(--color-secondary)]">
              {m.auto_z()}
            </span>
          </div>
          <div>
            <span class="block font-display text-xl font-bold text-white leading-tight">
              {m.auto_zafaf()} <span class="text-[var(--color-primary)]">{m.auto_world()}</span>
            </span>
          </div>
        </a>

        <p class="text-[var(--color-primary)] font-display text-lg italic mb-3">
          {m.footer_tagline()}
        </p>
        <p class="text-sm text-white/60 leading-relaxed mb-4">
          {m.footer_description()}
        </p>

        <!-- Contact Information -->
        <div class="mb-6 space-y-2.5 text-sm text-white/70">
          <div class="flex items-center gap-2.5">
            <svg viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4 text-[var(--color-primary)] shrink-0">
              <path d="M3 4a2 2 0 00-2 2v1.161l8.441 4.221a1.25 1.25 0 001.118 0L19 7.162V6a2 2 0 00-2-2H3z" />
              <path d="M19 8.839l-7.77 3.885a2.75 2.75 0 01-2.46 0L1 8.839V14a2 2 0 002 2h14a2 2 0 002-2V8.839z" />
            </svg>
            <a href="mailto:contact@zafafworld.net" class="hover:text-[var(--color-primary)] transition-colors">contact@zafafworld.net</a>
          </div>
          <div class="flex items-center gap-2.5">
            <svg viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4 text-[var(--color-primary)] shrink-0">
              <path d="M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z" />
            </svg>
            <a href="tel:+966592112517" class="hover:text-[var(--color-primary)] transition-colors" dir="ltr">+966 59 211 2517</a>
          </div>
        </div>

        <!-- Social Links -->
        <div>
          <p class="text-xs uppercase tracking-widest text-white/40 mb-3">{m.footer_followUs()}</p>
          <div class="flex items-center gap-3">
            {#each socialLinks as social}
              <a
                href={social.href}
                aria-label={social.label}
                class="w-11 h-11 rounded-lg bg-white/10 flex items-center justify-center text-white/60 hover:bg-[var(--color-primary)] hover:text-[var(--color-secondary)] transition-all duration-200"
              >
                {@html social.icon}
              </a>
            {/each}
          </div>
        </div>
      </div>

      <!-- Quick Links -->
      <div>
        <h3 class="text-sm uppercase tracking-widest text-white/40 mb-6">{m.footer_quickLinks()}</h3>
        <ul class="space-y-3" role="list">
          {#each quickLinks as link}
            <li>
              <a href={link.href} class="text-sm text-white/70 hover:text-[var(--color-primary)] transition-colors">
                {link.label}
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Support -->
      <div>
        <h3 class="text-sm uppercase tracking-widest text-white/40 mb-6">{m.footer_support()}</h3>
        <ul class="space-y-3" role="list">
          {#each supportLinks as link}
            <li>
              <a href={link.href} class="text-sm text-white/70 hover:text-[var(--color-primary)] transition-colors">
                {link.label}
              </a>
            </li>
          {/each}
          {#each legalLinks as link}
            <li>
              <a href={link.href} class="text-sm text-white/70 hover:text-[var(--color-primary)] transition-colors">
                {link.label}
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Newsletter -->
      <div>
        <h3 class="text-sm uppercase tracking-widest text-white/40 mb-2">{m.footer_newsletter_title()}</h3>
        <p class="text-sm text-white/60 mb-4">{m.footer_newsletter_subtitle()}</p>

        {#if subscribed}
          <div class="bg-[var(--color-primary)]/20 border border-[var(--color-primary)]/30 rounded-lg p-4 text-sm text-[var(--color-primary)]">
            ✓ {m.auto_subscribed_successfu()}
          </div>
        {:else}
          <form onsubmit={handleSubscribe} novalidate>
            <div class="flex flex-col gap-3">
              <label for="footer-email" class="sr-only">{m.footer_newsletter_placeholder()}</label>
              <input
                id="footer-email"
                type="email"
                bind:value={email}
                placeholder={m.footer_newsletter_placeholder()}
                required
                class="
                  w-full rounded-lg bg-white/10 border border-white/20 px-4 py-3
                  text-sm text-white placeholder:text-white/40
                  focus:outline-none focus:border-[var(--color-primary)] focus:ring-1 focus:ring-[var(--color-primary)]
                  transition-colors
                "
              />
              <button
                type="submit"
                class="w-full rounded-lg bg-[var(--color-primary)] text-[var(--color-secondary)] px-4 py-3 text-sm font-semibold hover:bg-[var(--color-primary-dark)] transition-colors cursor-pointer"
              >
                {m.footer_newsletter_submit()}
              </button>
            </div>
          </form>
        {/if}
      </div>

    </div>
  </div>

  <!-- Bottom Bar -->
  <div class="border-t border-white/10">
    <div class="container-page py-6 flex flex-col sm:flex-row items-center justify-between gap-4">
      <p class="text-xs text-white/40">
        © {currentYear} {m.meta_siteName()}. {m.footer_allRights()}
      </p>
      <div class="flex items-center gap-6">
        {#each legalLinks.slice(0, 2) as link}
          <a href={link.href} class="text-xs text-white/40 hover:text-[var(--color-primary)] transition-colors">
            {link.label}
          </a>
        {/each}
      </div>
    </div>
  </div>
</footer>
