<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Activity, AlertCircle } from 'lucide-svelte';
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.monitoring')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'مراقبة النظام' : 'System Monitoring'}</p>
    </div>
  </div>

  <div class="card" style="text-align: center; padding: 60px 20px; margin-top: 20px;">
    <div style="display: flex; justify-content: center; margin-bottom: 20px; color: var(--text-ghost);">
      <AlertCircle size={48} />
    </div>
    <h2 style="margin-bottom: 12px;">{$lang === 'ar' ? 'واجهة المراقبة قيد التطوير' : 'Live Monitoring Pending Implementation'}</h2>
    <p style="color: var(--text-secondary); max-width: 500px; margin: 0 auto 24px; line-height: 1.6;">
      {$lang === 'ar' 
        ? 'لم يتم ربط لوحة المراقبة بمصادر البيانات الحية بعد. لا توجد بيانات وهمية هنا. للحصول على إحصائيات النظام الحالية، يرجى مراجعة نقطة النهاية للعدادات الخام.' 
        : 'The real-time monitoring dashboard is not yet wired to live data sources. Zero fake data is presented here. For current system counters, please see the raw Prometheus metrics endpoint.'}
    </p>
    <a href="/api/v1/metrics" target="_blank" class="btn btn-primary" style="display: inline-flex; align-items: center; gap: 8px;">
      <Activity size={16} />
      {$lang === 'ar' ? 'عرض العدادات الخام (/api/v1/metrics)' : 'View Raw Metrics (/api/v1/metrics)'}
    </a>
  </div>
</div>
