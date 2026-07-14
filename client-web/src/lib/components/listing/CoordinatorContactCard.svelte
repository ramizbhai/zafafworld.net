<script lang="ts">
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { Phone, MessageCircle, Mail, User, Smartphone, Sparkles, ShieldCheck } from 'lucide-svelte';

  let { coordinator, vendor } = $props<{
    coordinator?: {
      nameAr: string | null;
      nameEn: string | null;
      phone: string | null;
      whatsapp: string | null;
      email: string | null;
      mobile: string | null;
      avatar: string | null;
      gender?: string | null;
    };
    vendor?: any;
  }>();

  const isAr = $derived(getLocale() === 'ar');

  // Resolve effective coordinator details (fall back to vendor brand phone/email if product coordinator is missing)
  const name = $derived(
    coordinator ? (isAr ? (coordinator.nameAr || coordinator.nameEn) : (coordinator.nameEn || coordinator.nameAr)) : null
  );
  const phone = $derived(coordinator?.phone || vendor?.phone || null);
  const whatsapp = $derived(coordinator?.whatsapp || vendor?.phone || null);
  const email = $derived(coordinator?.email || vendor?.email || null);
  const mobile = $derived(coordinator?.mobile || null);

  const hasContactInfo = $derived(!!(name || phone || whatsapp || email || mobile));

  function cleanPhone(num: string | null): string {
    if (!num) return '';
    return num.replace(/[^\d+]/g, '');
  }
</script>

{#if hasContactInfo}
  <section class="relative overflow-hidden rounded-2xl bg-white/80 backdrop-blur-md border border-slate-200/70 p-6 sm:p-8 shadow-sm hover:shadow-md transition-all duration-300">
    <!-- Decorative matte glassmorphism background glow -->
    <div class="absolute -right-16 -bottom-16 w-56 h-56 bg-amber-500/10 rounded-full blur-3xl pointer-events-none"></div>
    <div class="absolute -left-16 -top-16 w-56 h-56 bg-blue-500/5 rounded-full blur-3xl pointer-events-none"></div>

    <div class="relative z-10">
      <!-- Top Header -->
      <div class="flex items-center justify-between mb-6 border-b border-slate-100 pb-4">
        <div class="flex items-center gap-3.5">
          <div class="p-3 bg-gradient-to-br from-amber-50 to-amber-100/80 text-amber-600 rounded-2xl border border-amber-200/50 shadow-2xs">
            <User size={22} />
          </div>
          <div>
            <h3 class="font-display text-lg font-bold text-slate-900 leading-snug">
              {isAr ? 'منسق الحجوزات والاستفسارات' : 'Booking & Inquiries Coordinator'}
            </h3>
            <p class="text-xs font-medium text-slate-500 mt-0.5">
              {isAr ? 'تواصل مباشر مع المسؤول عن الخدمة' : 'Direct contact with event coordinator'}
            </p>
          </div>
        </div>
        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200/60 shadow-2xs">
          <Sparkles size={13} class="text-emerald-500" /> {isAr ? 'مباشر' : 'Direct'}
        </span>
      </div>

      {#if name}
        <div class="mb-6 bg-slate-50/80 p-4 rounded-xl border border-slate-100 flex items-center justify-between">
          <div>
            <span class="text-xs font-semibold uppercase tracking-wider text-slate-400 block mb-0.5">
              {isAr ? 'اسم المنسق' : 'Coordinator Name'}
            </span>
            <p class="text-lg font-bold text-slate-900">{name}</p>
          </div>
          <div class="hidden sm:flex items-center gap-1.5 text-xs text-slate-500 bg-white px-3 py-1.5 rounded-lg border border-slate-200/60 shadow-2xs">
            <ShieldCheck size={14} class="text-amber-500" />
            <span>{isAr ? 'معتمد' : 'Verified'}</span>
          </div>
        </div>
      {/if}

      <!-- Action CTA Buttons -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5 mb-6">
        {#if whatsapp}
          <a
            href={`https://wa.me/${cleanPhone(whatsapp)}`}
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center justify-center gap-2.5 px-5 py-3.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white font-semibold shadow-sm hover:shadow transition-all duration-200 text-sm active:scale-[0.99]"
          >
            <MessageCircle size={18} />
            <span>{isAr ? 'تواصل عبر واتساب' : 'WhatsApp Contact'}</span>
          </a>
        {/if}

        {#if phone}
          <a
            href={`tel:${cleanPhone(phone)}`}
            class="flex items-center justify-center gap-2.5 px-5 py-3.5 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-semibold shadow-sm hover:shadow transition-all duration-200 text-sm active:scale-[0.99]"
          >
            <Phone size={18} class="text-amber-400" />
            <span>{isAr ? 'اتصال تلفوني' : 'Call Phone'}</span>
          </a>
        {/if}
      </div>

      <!-- Secondary details list -->
      {#if (mobile && mobile !== phone) || email}
        <div class="space-y-2.5 pt-4 border-t border-slate-100 text-xs sm:text-sm text-slate-600">
          {#if mobile && mobile !== phone}
            <div class="flex items-center justify-between p-2.5 rounded-lg hover:bg-slate-50/80 transition-colors">
              <span class="text-slate-500 flex items-center gap-2 font-medium">
                <Smartphone size={15} class="text-amber-500" /> {isAr ? 'جوال إضافي:' : 'Additional Mobile:'}
              </span>
              <a href={`tel:${cleanPhone(mobile)}`} class="font-mono font-semibold text-slate-800 hover:text-amber-600 transition-colors" dir="ltr">{mobile}</a>
            </div>
          {/if}

          {#if email}
            <div class="flex items-center justify-between p-2.5 rounded-lg hover:bg-slate-50/80 transition-colors">
              <span class="text-slate-500 flex items-center gap-2 font-medium">
                <Mail size={15} class="text-amber-500" /> {isAr ? 'البريد الإلكتروني:' : 'Email Address:'}
              </span>
              <a href={`mailto:${email}`} class="font-mono font-semibold text-slate-800 hover:text-amber-600 transition-colors">{email}</a>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </section>
{/if}
