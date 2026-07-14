<script lang="ts">
    import { Star, BarChart3 } from 'lucide-svelte';
    import { getRatingPercent } from '../../services/dashboard.service';

    let { i18n, metrics } = $props<{
        i18n: any;
        metrics: any;
    }>();
</script>

<div class="card ratings-card">
    <div class="ratings-header">
        <div class="ratings-score-block">
            <div class="ratings-big-score">
                {metrics.avg_overall > 0 ? metrics.avg_overall.toFixed(1) : '—'}
            </div>
            <div class="ratings-stars">
                {#each Array(5) as _, i}
                    <Star
                        size={14}
                        fill={i < Math.round(metrics.avg_overall) ? 'hsl(40, 96%, 55%)' : 'none'}
                        stroke={i < Math.round(metrics.avg_overall) ? 'hsl(40, 96%, 55%)' : '#cbd5e1'}
                    />
                {/each}
            </div>
            <span class="ratings-count">
                {metrics.total_reviews}
                {i18n.locale === 'ar' ? ' تقييم' : ' reviews'}
            </span>
        </div>
        <h3 class="ratings-title">
            {i18n.locale === 'ar' ? 'مؤشر أداء التقييمات' : 'Review Performance'}
        </h3>
    </div>

    <div class="ratings-axes">
        <!-- Quality -->
        <div class="axis-row">
            <span class="axis-name">
                {i18n.locale === 'ar' ? 'جودة الخدمة' : 'Quality'}
            </span>
            <div class="progress-track">
                <div class="progress-fill" style="width: {getRatingPercent(metrics.avg_quality)}%"></div>
            </div>
            <span class="axis-score">
                {metrics.avg_quality > 0 ? metrics.avg_quality.toFixed(1) : '0.0'}
            </span>
        </div>
        <!-- Staff -->
        <div class="axis-row">
            <span class="axis-name">
                {i18n.locale === 'ar' ? 'كفاءة الفريق' : 'Staff'}
            </span>
            <div class="progress-track">
                <div class="progress-fill gold" style="width: {getRatingPercent(metrics.avg_staff)}%"></div>
            </div>
            <span class="axis-score">
                {metrics.avg_staff > 0 ? metrics.avg_staff.toFixed(1) : '0.0'}
            </span>
        </div>
        <!-- Communication -->
        <div class="axis-row">
            <span class="axis-name">
                {i18n.locale === 'ar' ? 'سرعة التواصل' : 'Communication'}
            </span>
            <div class="progress-track">
                <div class="progress-fill" style="width: {getRatingPercent(metrics.avg_communication)}%; background: hsl(258, 80%, 60%);"></div>
            </div>
            <span class="axis-score">
                {metrics.avg_communication > 0 ? metrics.avg_communication.toFixed(1) : '0.0'}
            </span>
        </div>
    </div>

    <a href="/dashboard/reviews" class="btn btn-outline btn-sm btn-block view-reviews-btn">
        <BarChart3 size={14} />
        {i18n.locale === 'ar' ? 'عرض التقييمات التفصيلية' : 'View Detailed Reviews'}
    </a>
</div>
