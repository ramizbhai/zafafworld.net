<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  const values = $derived([
    { icon: '💎', titleAr: 'الجودة', titleEn: 'Quality', descAr: 'نضمن أعلى معايير الجودة في كل قاعة', descEn: 'We ensure the highest quality standards in every venue' },
    { icon: '🤝', titleAr: 'الثقة', titleEn: 'Trust', descAr: 'شفافية كاملة في الأسعار والخدمات', descEn: 'Full transparency in pricing and services' },
    { icon: '⚡', titleAr: 'السرعة', titleEn: 'Speed', descAr: 'حجز فوري وتأكيد في ثوانٍ', descEn: 'Instant booking and confirmation in seconds' },
    { icon: '💛', titleAr: 'الاهتمام', titleEn: 'Care', descAr: 'فريق دعم على مدار الساعة لخدمتكم', descEn: '24/7 support team at your service' },
  ]);

  const team = [
    { nameAr: 'عبدالرحمن الشمري', nameEn: 'Abdulrahman Al-Shamri', roleAr: 'المؤسس والرئيس التنفيذي', roleEn: 'Founder & CEO', avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=200&q=80' },
    { nameAr: 'سارة العتيبي', nameEn: 'Sarah Al-Otaibi', roleAr: 'مديرة العمليات', roleEn: 'Chief Operations Officer', avatar: 'https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=200&q=80' },
    { nameAr: 'محمد الحربي', nameEn: 'Mohammed Al-Harbi', roleAr: 'مدير التقنية', roleEn: 'Chief Technology Officer', avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=200&q=80' },
  ];
</script>

<svelte:head>
  <title>{m.about_title()} — {m.meta_siteName()}</title>
</svelte:head>

<!-- Hero -->
<div class="relative overflow-hidden bg-[var(--color-secondary)] text-white">
  <div class="absolute inset-0 opacity-10">
    <div class="absolute top-0 start-0 w-96 h-96 rounded-full bg-[var(--color-primary)] -translate-x-1/2 -translate-y-1/2"></div>
    <div class="absolute bottom-0 end-0 w-64 h-64 rounded-full bg-[var(--color-primary)] translate-x-1/2 translate-y-1/2"></div>
  </div>
  <div class="container-page py-24 relative z-10">
    <span class="divider-gold"></span>
    <h1 class="font-display text-4xl sm:text-5xl font-bold mt-6 mb-4">{m.about_title()}</h1>
    <p class="text-xl text-white/70 max-w-2xl">{m.about_subtitle()}</p>
  </div>
</div>

<!-- Mission & Vision -->
<section class="py-20 bg-white" aria-labelledby="mission-title">
  <div class="container-page">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-12">
      <div class="p-8 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-alt)]">
        <div class="text-4xl mb-4" aria-hidden="true">🎯</div>
        <h2 id="mission-title" class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-4">
          {m.about_mission_title()}
        </h2>
        <p class="text-[var(--color-muted)] leading-relaxed">{m.about_mission_description()}</p>
      </div>
      <div class="p-8 rounded-2xl border border-[var(--color-primary)] bg-[var(--color-primary)]/5">
        <div class="text-4xl mb-4" aria-hidden="true">🌟</div>
        <h2 class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-4">
          {m.about_vision_title()}
        </h2>
        <p class="text-[var(--color-muted)] leading-relaxed">{m.about_vision_description()}</p>
      </div>
    </div>
  </div>
</section>

<!-- Values -->
<section class="py-20 bg-[var(--color-surface-alt)]" aria-labelledby="values-title">
  <div class="container-page">
    <div class="text-center mb-12">
      <span class="divider-gold mx-auto"></span>
      <h2 id="values-title" class="font-display text-3xl font-bold text-[var(--color-secondary)] mt-6">{m.about_values()}</h2>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
      {#each values as val}
        <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] text-center hover:shadow-[var(--shadow-md)] transition-shadow">
          <div class="text-4xl mb-4" aria-hidden="true">{val.icon}</div>
          <h3 class="font-display text-lg font-semibold text-[var(--color-secondary)] mb-2">
            {getLocalizedField(val, 'title', getLocale())}
          </h3>
          <p class="text-sm text-[var(--color-muted)]">
            {getLocalizedField(val, 'desc', getLocale())}
          </p>
        </div>
      {/each}
    </div>
  </div>
</section>

<!-- Team -->
<section class="py-20 bg-white" aria-labelledby="team-title">
  <div class="container-page">
    <div class="text-center mb-12">
      <span class="divider-gold mx-auto"></span>
      <h2 id="team-title" class="font-display text-3xl font-bold text-[var(--color-secondary)] mt-6">{m.about_team()}</h2>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-8 max-w-3xl mx-auto">
      {#each team as member}
        <div class="text-center group">
          <div class="w-28 h-28 rounded-2xl mx-auto mb-4 overflow-hidden border-2 border-[var(--color-border)] group-hover:border-[var(--color-primary)] transition-colors">
            <img src={member.avatar} alt={getLocalizedField(member, 'name', getLocale())} class="w-full h-full object-cover" loading="lazy" />
          </div>
          <h3 class="font-semibold text-[var(--color-secondary)]">
            {getLocalizedField(member, 'name', getLocale())}
          </h3>
          <p class="text-sm text-[var(--color-muted)] mt-1">
            {getLocalizedField(member, 'role', getLocale())}
          </p>
        </div>
      {/each}
    </div>
  </div>
</section>
