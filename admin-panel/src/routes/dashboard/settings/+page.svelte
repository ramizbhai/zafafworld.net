P<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Settings, Save, CheckCircle } from 'lucide-svelte';
  import { enhance } from '$app/forms';

  let { data } = $props();

  let activeTab = $state('general');
  const tabs = [
    { id: 'general', label_ar: 'عام', label_en: 'General' },
    { id: 'security', label_ar: 'الأمان', label_en: 'Security' },
  ];

  // Dynamic Settings States from loaded server dataset
  let platformName = $state('');
  let platformEmail = $state('');
  let defaultCurrency = $state('SAR');
  let commissionRate = $state(10.0);
  let sessionTimeout = $state(60);
  let maxLoginAttempts = $state(5);

  let enableTabby = $state(false);
  let enableReviews = $state(false);
  let enableAiModeration = $state(false);
  let maintenanceMode = $state(false);
  let enableWhatsapp = $state(false);

  $effect(() => {
    platformName = data.settings?.platform_name ?? 'ZafafWorld';
    platformEmail = data.settings?.platform_email ?? 'admin@zafafworld.net';
    defaultCurrency = data.settings?.default_currency ?? 'SAR';
    commissionRate = Number(data.settings?.platform_commission_rate ?? '10.0');
    sessionTimeout = Number(data.settings?.session_timeout ?? '60');
    maxLoginAttempts = Number(data.settings?.max_login_attempts ?? '5');
    enableTabby = data.settings?.enable_tabby === 'true';
    enableReviews = data.settings?.enable_reviews === 'true';
    enableAiModeration = data.settings?.enable_ai_moderation === 'true';
    maintenanceMode = data.settings?.maintenance_mode === 'true';
    enableWhatsapp = data.settings?.enable_whatsapp === 'true';
  });

  let isSubmitting = $state(false);
  let showSuccess = $state(false);
  let errorMessage = $state('');
</script>

<div class="fade-in">
  <!-- Status Toasts -->
  {#if showSuccess}
    <div class="alert alert-success" style="position: fixed; top: 20px; right: 20px; z-index: 1000; box-shadow: 0 4px 12px rgba(0,0,0,0.15); display: flex; align-items: center; gap: 8px;">
      <CheckCircle size={18} />
      <span>{$lang === 'ar' ? 'تم حفظ التغييرات بنجاح' : 'Changes saved successfully'}</span>
    </div>
  {/if}

  {#if errorMessage}
    <div class="alert alert-danger" style="position: fixed; top: 20px; right: 20px; z-index: 1000; box-shadow: 0 4px 12px rgba(0,0,0,0.15);">
      <span>{errorMessage}</span>
    </div>
  {/if}

  <form method="POST" action="?/saveSettings" use:enhance={() => {
    isSubmitting = true;
    errorMessage = '';
    return async ({ result, update }) => {
      isSubmitting = false;
      if (result.type === 'success') {
        showSuccess = true;
        setTimeout(() => { showSuccess = false; }, 3000);
      } else {
        errorMessage = $lang === 'ar' ? 'فشل حفظ الإعدادات' : 'Failed to save settings';
      }
      update({ reset: false });
    };
  }}>
    <!-- Hidden fields to capture configuration state -->
    <input type="hidden" name="platform_name" value={platformName} />
    <input type="hidden" name="platform_email" value={platformEmail} />
    <input type="hidden" name="default_currency" value={defaultCurrency} />
    <input type="hidden" name="platform_commission_rate" value={commissionRate} />
    <input type="hidden" name="session_timeout" value={sessionTimeout} />
    <input type="hidden" name="max_login_attempts" value={maxLoginAttempts} />
    <input type="hidden" name="enable_tabby" value={enableTabby ? 'true' : 'false'} />
    <input type="hidden" name="enable_reviews" value={enableReviews ? 'true' : 'false'} />
    <input type="hidden" name="enable_ai_moderation" value={enableAiModeration ? 'true' : 'false'} />
    <input type="hidden" name="maintenance_mode" value={maintenanceMode ? 'true' : 'false'} />
    <input type="hidden" name="enable_whatsapp" value={enableWhatsapp ? 'true' : 'false'} />

    <div class="page-header">
      <div class="page-header-left">
        <h1 class="page-title">{$t('nav.settings')}</h1>
        <p class="page-subtitle">{$lang === 'ar' ? 'إعدادات وتكوينات المنصة الشاملة' : 'Comprehensive platform settings and configurations'}</p>
      </div>
      <button type="submit" class="btn btn-gold btn-sm" disabled={isSubmitting} onclick={(e) => { if (!confirm($lang === 'ar' ? 'هل أنت متأكد من حفظ التغييرات العامة؟' : 'Are you sure you want to save these global changes?')) e.preventDefault(); }}>
        <Save size={14} /> 
        {isSubmitting ? ($lang === 'ar' ? 'جاري الحفظ...' : 'Saving...') : ($lang === 'ar' ? 'حفظ التغييرات' : 'Save Changes')}
      </button>
    </div>

    <!-- Tabs -->
    <div class="tabs-bar" style="margin-bottom: 20px; width:100%;">
      {#each tabs as tab}
        <button type="button" class="tab-btn" class:active={activeTab === tab.id} onclick={() => activeTab = tab.id}>
          {$lang === 'ar' ? tab.label_ar : tab.label_en}
        </button>
      {/each}
    </div>

    {#if activeTab === 'general'}
      <div class="settings-grid">
        <div class="card settings-section">
          <div class="card-header">
            <span class="table-title">{$lang === 'ar' ? 'معلومات المنصة' : 'Platform Information'}</span>
          </div>
          <div class="card-body">
            <div class="form-group">
              <label class="form-label" for="platform-name">{$lang === 'ar' ? 'اسم المنصة' : 'Platform Name'}</label>
              <input class="form-input" id="platform-name" bind:value={platformName} type="text" required />
            </div>
            <div class="form-group">
              <label class="form-label" for="platform-email">{$lang === 'ar' ? 'البريد الإلكتروني الرسمي' : 'Official Email'}</label>
              <input class="form-input" id="platform-email" bind:value={platformEmail} type="email" required />
            </div>
            <div class="form-group">
              <label class="form-label" for="default-currency">{$lang === 'ar' ? 'العملة الافتراضية' : 'Default Currency'}</label>
              <select class="form-select" id="default-currency" bind:value={defaultCurrency}>
                <option value="SAR">SAR — ريال سعودي</option>
                <option value="AED">AED — درهم إماراتي</option>
                <option value="USD">USD — Dollar</option>
              </select>
            </div>
            <div class="form-group">
              <label class="form-label" for="commission-rate">{$lang === 'ar' ? 'نسبة العمولة الافتراضية (%)' : 'Default Commission Rate (%)'}</label>
              <input class="form-input" id="commission-rate" bind:value={commissionRate} type="number" min="0" max="100" step="0.1" required />
              <p class="form-hint">{$lang === 'ar' ? 'النسبة المئوية من قيمة كل حجز' : 'Percentage of each booking value'}</p>
            </div>
          </div>
        </div>

        <!-- Feature Flags -->
        <div class="card settings-section">
          <div class="card-header">
            <span class="table-title">{$lang === 'ar' ? 'ميزات المنصة' : 'Feature Flags'}</span>
          </div>
          <div class="card-body" style="padding: 8px 0;">
            <div class="feature-flag-item">
              <div class="flag-info">
                <span class="flag-label">{$lang === 'ar' ? 'تفعيل التقسيط (Tabby)' : 'Enable Installments (Tabby)'}</span>
              </div>
              <button
                type="button"
                class="toggle-btn"
                class:toggle-on={enableTabby}
                onclick={() => enableTabby = !enableTabby}
                role="switch"
                aria-checked={enableTabby}
                aria-label="enable_tabby"
              >
                <div class="toggle-knob"></div>
              </button>
            </div>

            <div class="feature-flag-item">
              <div class="flag-info">
                <span class="flag-label">{$lang === 'ar' ? 'تفعيل نظام التقييمات' : 'Enable Review System'}</span>
              </div>
              <button
                type="button"
                class="toggle-btn"
                class:toggle-on={enableReviews}
                onclick={() => enableReviews = !enableReviews}
                role="switch"
                aria-checked={enableReviews}
                aria-label="enable_reviews"
              >
                <div class="toggle-knob"></div>
              </button>
            </div>

            <div class="feature-flag-item">
              <div class="flag-info">
                <span class="flag-label">{$lang === 'ar' ? 'مراجعة المحتوى بالذكاء الاصطناعي' : 'AI Content Moderation'}</span>
              </div>
              <button
                type="button"
                class="toggle-btn"
                class:toggle-on={enableAiModeration}
                onclick={() => enableAiModeration = !enableAiModeration}
                role="switch"
                aria-checked={enableAiModeration}
                aria-label="enable_ai_moderation"
              >
                <div class="toggle-knob"></div>
              </button>
            </div>

            <div class="feature-flag-item">
              <div class="flag-info">
                <span class="flag-label">{$lang === 'ar' ? 'وضع الصيانة' : 'Maintenance Mode'}</span>
              </div>
              <button
                type="button"
                class="toggle-btn"
                class:toggle-on={maintenanceMode}
                onclick={() => maintenanceMode = !maintenanceMode}
                role="switch"
                aria-checked={maintenanceMode}
                aria-label="maintenance_mode"
              >
                <div class="toggle-knob"></div>
              </button>
            </div>

            <div class="feature-flag-item">
              <div class="flag-info">
                <span class="flag-label">{$lang === 'ar' ? 'إشعارات واتساب' : 'WhatsApp Notifications'}</span>
              </div>
              <button
                type="button"
                class="toggle-btn"
                class:toggle-on={enableWhatsapp}
                onclick={() => enableWhatsapp = !enableWhatsapp}
                role="switch"
                aria-checked={enableWhatsapp}
                aria-label="enable_whatsapp"
              >
                <div class="toggle-knob"></div>
              </button>
            </div>
          </div>
        </div>
      </div>
    {:else if activeTab === 'security'}
      <div class="card settings-section">
        <div class="card-header"><span class="table-title">{$lang === 'ar' ? 'إعدادات الأمان' : 'Security Settings'}</span></div>
        <div class="card-body">
          <div class="form-group">
            <label class="form-label" for="session-timeout">{$lang === 'ar' ? 'مهلة الجلسة (دقائق)' : 'Session Timeout (minutes)'}</label>
            <input class="form-input" id="session-timeout" bind:value={sessionTimeout} type="number" style="max-width:200px;" required />
          </div>
          <div class="form-group">
            <label class="form-label" for="max-login-attempts">{$lang === 'ar' ? 'الحد الأقصى لمحاولات تسجيل الدخول' : 'Max Login Attempts'}</label>
            <input class="form-input" id="max-login-attempts" bind:value={maxLoginAttempts} type="number" style="max-width:200px;" required />
          </div>
          </div>
        </div>
    {/if}
  </form>
</div>

<style>
  .settings-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .feature-flag-item {
    display: flex; align-items: center; justify-content: space-between;
    padding: 13px 20px; border-bottom: 1px solid var(--glass-border);
  }
  .feature-flag-item:last-child { border-bottom: none; }
  .flag-info { flex: 1; }
  .flag-label { font-size: 13.5px; font-weight: 600; color: var(--text-primary); }
  .toggle-btn {
    width: 44px; height: 24px; border-radius: 999px;
    background: var(--bg-float); border: 1.5px solid var(--glass-border);
    position: relative; cursor: pointer; flex-shrink: 0;
    transition: all 220ms var(--ease-smooth);
  }
  .toggle-btn.toggle-on {
    background: var(--gold); border-color: var(--gold);
  }
  .toggle-knob {
    position: absolute; top: 2px; inset-inline-start: 2px;
    width: 16px; height: 16px; border-radius: 50%;
    background: #fff;
    transition: inset-inline-start 220ms var(--ease-smooth);
    box-shadow: 0 1px 4px rgba(0,0,0,0.3);
  }
  .toggle-btn.toggle-on .toggle-knob {
    inset-inline-start: calc(100% - 18px);
  }
  @media (max-width: 900px) { .settings-grid { grid-template-columns: 1fr; } }
</style>
