<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Activity, Server, Cpu, Database, Globe, AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-svelte';

  const services = [
    { name: 'API Gateway', status: 'operational', uptime: '99.98%', latency: '42ms', requests: '1.2M/day' },
    { name: 'PostgreSQL Primary', status: 'operational', uptime: '99.99%', latency: '8ms', requests: '4.8M/day' },
    { name: 'Redis Cache', status: 'operational', uptime: '100%', latency: '1ms', requests: '12M/day' },
    { name: 'File Storage (S3)', status: 'operational', uptime: '99.99%', latency: '120ms', requests: '280K/day' },
    { name: 'Email Service', status: 'degraded', uptime: '98.2%', latency: '840ms', requests: '45K/day' },
    { name: 'Payment Gateway', status: 'operational', uptime: '99.95%', latency: '220ms', requests: '8.2K/day' },
  ];

  const recentAlerts = [
    { time: '14:32', type: 'warning', msg_ar: 'ارتفاع طفيف في زمن الاستجابة لخدمة البريد', msg_en: 'Slight latency spike in email service' },
    { time: '12:18', type: 'info', msg_ar: 'اكتمل النسخ الاحتياطي التلقائي', msg_en: 'Automated backup completed successfully' },
    { time: '09:44', type: 'info', msg_ar: 'تحديث الشهادات الأمنية TLS بنجاح', msg_en: 'TLS certificates renewed successfully' },
    { time: '08:22', type: 'warning', msg_ar: 'مهلة طلب بحث معقد تجاوز 3 ثوان', msg_en: 'Complex search query timeout > 3s detected' },
  ];

  const systemMetrics = [
    { label_ar: 'معالج الخادم', label_en: 'CPU Usage', value: 34, unit: '%', color: 'var(--info)' },
    { label_ar: 'الذاكرة', label_en: 'Memory', value: 58, unit: '%', color: 'var(--gold)' },
    { label_ar: 'قاعدة البيانات', label_en: 'DB Pool', value: 22, unit: '%', color: 'var(--success)' },
    { label_ar: 'الشبكة الواردة', label_en: 'Network In', value: 67, unit: '%', color: 'var(--purple)' },
  ];

  function statusBadge(s: string) {
    if (s === 'operational') return 'badge badge-dot badge-success';
    if (s === 'degraded') return 'badge badge-dot badge-warning';
    return 'badge badge-dot badge-danger';
  }
  function statusLabel(s: string) {
    const map: Record<string,string> = { operational: $lang==='ar'?'يعمل':'Operational', degraded: $lang==='ar'?'أداء منخفض':'Degraded', down: $lang==='ar'?'متوقف':'Down' };
    return map[s] ?? s;
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.monitoring')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'مراقبة صحة النظام والأداء في الوقت الفعلي' : 'Real-time system health and performance monitoring'}</p>
    </div>
    <button class="btn btn-outline btn-sm"><RefreshCw size={14} /> {$lang === 'ar' ? 'تحديث' : 'Refresh'}</button>
  </div>

  <!-- System metrics -->
  <div class="metrics-grid">
    {#each systemMetrics as m}
      <div class="metric-card card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px">
          <span class="mini-stat-label">{$lang === 'ar' ? m.label_ar : m.label_en}</span>
          <span style="font-size:20px; font-weight:800; color:{m.color}">{m.value}{m.unit}</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" style="width:{m.value}%; background:{m.color};"></div>
        </div>
        <div style="display:flex; justify-content:space-between; margin-top:6px; font-size:11px; color:var(--text-ghost)">
          <span>0{m.unit}</span>
          <span>100{m.unit}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="mon-grid">
    <!-- Services table -->
    <div class="table-container">
      <div class="table-head-bar">
        <span class="table-title">{$lang === 'ar' ? 'حالة الخدمات' : 'Service Status'}</span>
        <div class="status-dot status-live" style="margin-inline-start:auto"></div>
      </div>
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>{$lang === 'ar' ? 'الخدمة' : 'Service'}</th>
              <th>{$lang === 'ar' ? 'وقت التشغيل' : 'Uptime'}</th>
              <th>{$lang === 'ar' ? 'زمن الاستجابة' : 'Latency'}</th>
              <th>{$lang === 'ar' ? 'الطلبات' : 'Requests'}</th>
              <th>{$lang === 'ar' ? 'الحالة' : 'Status'}</th>
            </tr>
          </thead>
          <tbody>
            {#each services as svc}
              <tr>
                <td style="font-weight:600">{svc.name}</td>
                <td style="font-weight:700; color:var(--success)">{svc.uptime}</td>
                <td class="mono" style="font-size:12px">{svc.latency}</td>
                <td class="text-muted" style="font-size:12.5px">{svc.requests}</td>
                <td><span class={statusBadge(svc.status)}>{statusLabel(svc.status)}</span></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

    <!-- Recent Alerts -->
    <div class="card alert-card">
      <div class="card-header">
        <span class="table-title">{$lang === 'ar' ? 'التنبيهات الأخيرة' : 'Recent Alerts'}</span>
      </div>
      <div class="card-body" style="padding:0;">
        {#each recentAlerts as alert}
          <div class="alert-item" class:alert-warn={alert.type==='warning'}>
            <span class="alert-time mono">{alert.time}</span>
            <div class="alert-icon-sm" class:icon-warn={alert.type==='warning'} class:icon-info-sm={alert.type==='info'}>
              {#if alert.type === 'warning'}
                <AlertTriangle size={13} />
              {:else}
                <CheckCircle2 size={13} />
              {/if}
            </div>
            <span class="alert-msg">{$lang === 'ar' ? alert.msg_ar : alert.msg_en}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .metrics-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .metric-card { padding: 16px 18px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-ghost); }
  .mon-grid { display: grid; grid-template-columns: 3fr 2fr; gap: 16px; }
  .alert-item {
    display: flex; align-items: center; gap: 10px;
    padding: 11px 18px; border-bottom: 1px solid var(--glass-border);
    font-size: 12.5px; color: var(--text-secondary);
  }
  .alert-item:last-child { border-bottom: none; }
  .alert-item.alert-warn { background: var(--warning-dim); }
  .alert-time { font-family: var(--font-mono); font-size: 11px; color: var(--text-ghost); flex-shrink: 0; }
  .alert-icon-sm { width: 22px; height: 22px; border-radius: 5px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .icon-warn { background: var(--warning-dim); color: var(--warning); }
  .icon-info-sm { background: var(--success-dim); color: var(--success); }
  .alert-msg { flex: 1; line-height: 1.4; }
  @media (max-width: 1100px) { .metrics-grid { grid-template-columns: repeat(2, 1fr); } .mon-grid { grid-template-columns: 1fr; } }
</style>
