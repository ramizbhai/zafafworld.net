<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { TrendingUp, TrendingDown, Minus } from 'lucide-svelte';

    interface Props {
        label: string;
        value: string | number;
        icon: string;
        color?: 'teal' | 'blue' | 'orange' | 'red' | 'purple' | 'gold';
        trend?: number;
        trendLabel?: string;
        prefix?: string;
        suffix?: string;
        premiumBorder?: boolean;
    }

    let {
        label,
        value,
        icon,
        color = 'teal',
        trend,
        trendLabel,
        prefix = '',
        suffix = '',
        premiumBorder = false
    }: Props = $props();

    const i18n = getI18n();

    let trendDir = $derived(
        trend === undefined ? 'neutral'
        : trend > 0 ? 'up'
        : trend < 0 ? 'down'
        : 'neutral'
    );

    let trendAbs = $derived(trend !== undefined ? Math.abs(trend) : 0);
</script>

<div class="stat-card card-hover" class:premium-border={premiumBorder}>
    <div class="stat-body">
        <div class="stat-label">{label}</div>
        <div class="stat-value">
            {#if prefix}<span class="stat-affix">{prefix}</span>{/if}
            {value}
            {#if suffix}<span class="stat-affix stat-suffix">{suffix}</span>{/if}
        </div>
        {#if trend !== undefined}
            <div class="stat-trend {trendDir}">
                {#if trendDir === 'up'}
                    <TrendingUp size={11} />
                {:else if trendDir === 'down'}
                    <TrendingDown size={11} />
                {:else}
                    <Minus size={11} />
                {/if}
                <span>{trendAbs}%</span>
                {#if trendLabel}<span class="trend-period">{trendLabel}</span>{/if}
            </div>
        {/if}
    </div>
    <div class="stat-icon icon-{color}">
        {icon}
    </div>
</div>

<style>
    .premium-border {
        border-top: 4px solid var(--color-secondary) !important;
    }
    .stat-affix {
        font-size: 0.55em;
        font-weight: 500;
        color: var(--text-sec);
        letter-spacing: 0;
    }
    .stat-suffix { margin-inline-start: 2px; }
    .trend-period {
        font-weight: 400;
        opacity: 0.75;
        font-size: 10px;
    }
</style>
