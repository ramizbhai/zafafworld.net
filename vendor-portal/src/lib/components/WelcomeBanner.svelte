<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { Sparkles } from 'lucide-svelte';

    interface Props {
        name: string;
    }

    let { name }: Props = $props();
    const i18n = getI18n();

    // Get hour-based greeting
    let greeting = $derived.by(() => {
        const hour = new Date().getHours();
        if (i18n.locale === 'ar') {
            if (hour < 12) return 'صباح الخير';
            if (hour < 17) return 'مساء الخير';
            return 'مساء الخير';
        } else {
            if (hour < 12) return 'Good morning';
            if (hour < 17) return 'Good afternoon';
            return 'Good evening';
        }
    });

    let subtitle = $derived(
        i18n.locale === 'ar'
            ? `مرحباً بك في لوحة تحكم ${name}. إليك ملخص يومك.`
            : `Welcome back, ${name}. Here's your performance overview for today.`
    );
</script>

<div class="welcome-banner">
    <div class="welcome-glow"></div>
    <div class="welcome-content">
        <div class="welcome-icon">
            <Sparkles size={20} />
        </div>
        <div class="welcome-text">
            <h2 class="welcome-greeting">
                {greeting}{i18n.locale === 'ar' ? '،' : ','} <span class="welcome-name">{name}</span>
            </h2>
            <p class="welcome-subtitle">{subtitle}</p>
        </div>
    </div>
    <div class="welcome-date">
        <span class="date-label">
            {new Date().toLocaleDateString(i18n.locale === 'ar' ? 'ar-SA' : 'en-US', {
                weekday: 'long',
                year: 'numeric',
                month: 'long',
                day: 'numeric'
            })}
        </span>
    </div>
</div>

<style>
    .welcome-banner {
        background: linear-gradient(135deg, hsl(162, 72%, 34%) 0%, hsl(162, 72%, 26%) 70%, hsl(162, 55%, 20%) 100%);
        border-radius: var(--radius-lg);
        padding: 22px 28px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        position: relative;
        overflow: hidden;
        box-shadow: 0 8px 24px -6px hsl(162, 72%, 30%, 0.35);
        margin-bottom: 4px;
    }

    .welcome-glow {
        position: absolute;
        width: 300px;
        height: 300px;
        background: radial-gradient(circle, rgba(255, 255, 255, 0.06) 0%, transparent 70%);
        top: -120px;
        inset-inline-start: -60px;
        pointer-events: none;
    }

    .welcome-banner::after {
        content: '';
        position: absolute;
        inset-inline-end: 0;
        top: 0;
        bottom: 0;
        width: 40%;
        background: linear-gradient(90deg, transparent, rgba(255,255,255,0.02));
        pointer-events: none;
    }

    .welcome-content {
        display: flex;
        align-items: center;
        gap: 14px;
        position: relative;
        z-index: 1;
    }

    .welcome-icon {
        width: 42px;
        height: 42px;
        border-radius: 12px;
        background: rgba(255, 255, 255, 0.12);
        display: flex;
        align-items: center;
        justify-content: center;
        color: hsl(40, 96%, 70%);
        flex-shrink: 0;
        border: 1px solid rgba(255, 255, 255, 0.10);
    }

    .welcome-text { display: flex; flex-direction: column; gap: 3px; }

    .welcome-greeting {
        font-size: 17px;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.90);
        margin: 0;
        letter-spacing: -0.2px;
    }

    .welcome-name { color: #fff; font-weight: 800; }

    .welcome-subtitle {
        font-size: 13px;
        color: rgba(255, 255, 255, 0.60);
        margin: 0;
        font-weight: 400;
    }

    .welcome-date {
        position: relative;
        z-index: 1;
        flex-shrink: 0;
    }

    .date-label {
        font-size: 12px;
        font-weight: 500;
        color: rgba(255, 255, 255, 0.55);
        display: block;
        text-align: var(--text-align);
    }

    @media (max-width: 600px) {
        .welcome-banner { padding: 18px 20px; flex-direction: column; align-items: flex-start; }
        .welcome-date { display: none; }
    }
</style>
