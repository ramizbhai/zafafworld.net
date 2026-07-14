<script lang="ts">
    import { enhance } from '$app/forms';
    import type { SubmitFunction } from '@sveltejs/kit';
    import { Lock, ArrowRight, ArrowLeft, ShieldCheck, Heart } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { page } from '$app/stores';
    import { uiStore } from '$lib/stores/ui.svelte';

    let { form } = $props();
    let password = $state('');
    let confirmPassword = $state('');
    let success = $state(false);

    const i18n = getI18n();

    // Reactively extract token from URL query params
    let token = $derived($page.url.searchParams.get('token') ?? '');

    const handleSubmit: SubmitFunction = () => {
        if (password.length < 8) {
            return () => {};
        }
        if (password !== confirmPassword) {
            return () => {};
        }

        uiStore.setLoading(true);
        return async ({ result, update }) => {
            uiStore.setLoading(false);
            if (result.type === 'success' && result.data?.success) {
                success = true;
            } else {
                await update();
            }
        };
    };
</script>

<svelte:head>
    <title>{i18n.locale === 'ar' ? 'تعيين كلمة السر الجديدة' : 'Reset Password'} | ZafafWorld Cockpit</title>
</svelte:head>

<div class="login-layout" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <!-- Left Hero Brand Panel -->
    <div class="brand-panel">
        <div class="glow-orb gold-orb"></div>
        <div class="glow-orb teal-orb"></div>
        
        <div class="panel-content">
            <div class="brand-header">
                <span class="gold-badge">ZAFAF COCKPIT</span>
                <h2>{i18n.locale === 'ar' ? 'تحديث حماية حسابك' : 'Secure Your Partner Account'}</h2>
                <p class="tagline">{i18n.locale === 'ar' ? 'اختر كلمة مرور قوية وجديدة لحماية لوحة التحكم والبيانات الخاصة بك.' : 'Choose a strong and secure new password to protect your partner cockpit and bookings.'}</p>
            </div>
            
            <div class="features-list">
                <div class="feature-item">
                    <div class="feature-icon">
                        <Heart size={20} />
                    </div>
                    <div class="feature-text">
                        <h3>{i18n.locale === 'ar' ? 'حماية شاملة للمبيعات' : 'Secure Pipelines'}</h3>
                        <p>{i18n.locale === 'ar' ? 'إدارتك لمعلومات العرسان والأسعار والقاعات محمية بنظام معايير أمان بنكية.' : 'Your customer details, prices, and venue listings are fully protected with banking-grade security.'}</p>
                    </div>
                </div>
                
                <div class="feature-item">
                    <div class="feature-icon">
                        <ShieldCheck size={20} />
                    </div>
                    <div class="feature-text">
                        <h3>{i18n.locale === 'ar' ? 'إلغاء فوري للجلسات القديمة' : 'Immediate Invalidation'}</h3>
                        <p>{i18n.locale === 'ar' ? 'عند تعيين كلمة المرور الجديدة، سيقوم النظام تلقائياً بتسجيل الخروج من كافة الأجهزة للحماية.' : 'Resetting your password immediately invalidates all active sessions across all other devices.'}</p>
                    </div>
                </div>
            </div>

            <div class="panel-footer">
                <p>{i18n.locale === 'ar' ? 'المنصة الأولى لإدارة قاعات ومزودي خدمات الأفراح.' : 'Welcome back to the region\'s elite wedding vendor platform.'}</p>
            </div>
        </div>
    </div>

    <!-- Right Login Card Panel -->
    <div class="form-panel">
        <div class="form-wrapper">
            {#if success}
                <div class="glassmorphic-card success-card animate-fade-in">
                    <div class="success-icon-container">
                        <ShieldCheck size={48} class="success-icon" />
                    </div>
                    <h1>{i18n.locale === 'ar' ? 'تم تعيين كلمة السر' : 'Password Updated'}</h1>
                    <p class="success-description">
                        {i18n.locale === 'ar'
                            ? 'لقد تم تحديث كلمة المرور الخاصة بك بنجاح. يمكنك الآن استخدامها لتسجيل الدخول.'
                            : 'Your partner cockpit password has been successfully updated. You can now log in securely.'}
                    </p>
                    <a href="/login" class="back-login-btn">
                        <ArrowLeft size={16} /> {i18n.locale === 'ar' ? 'تسجيل الدخول الآن' : 'Log In Securely'}
                    </a>
                </div>
            {:else if !token}
                <div class="glassmorphic-card success-card animate-fade-in" style="border-color: rgba(239, 68, 68, 0.2);">
                    <div class="success-icon-container" style="background: rgba(239, 68, 68, 0.08); border-color: rgba(239, 68, 68, 0.2); color: #ef4444;">
                        <span style="font-size: 2rem;">⚠️</span>
                    </div>
                    <h1 style="color: #b91c1c;">{i18n.locale === 'ar' ? 'رابط غير صالح' : 'Invalid Link'}</h1>
                    <p class="success-description">
                        {i18n.locale === 'ar'
                            ? 'رمز استعادة كلمة المرور هذا غير صالح أو منتهي الصلاحية. يرجى طلب رابط جديد.'
                            : 'The password recovery token you followed is invalid, expired, or missing. Please request a new recovery link.'}
                    </p>
                    <a href="/forgot-password" class="back-login-btn" style="background: #ef4444; box-shadow: 0 4px 12px rgba(239, 68, 68, 0.25);">
                        {i18n.locale === 'ar' ? 'طلب رابط جديد' : 'Request Recovery Link'}
                    </a>
                </div>
            {:else}
                <div class="login-header">
                    <h1>{i18n.locale === 'ar' ? 'كلمة سر جديدة' : 'New Security Key'}</h1>
                    <p class="subtitle font-cairo">
                        {i18n.locale === 'ar' ? 'اختر كلمة سر قوية (8 أحرف على الأقل) وقم بتأكيدها بالأسفل.' : 'Enter and confirm a highly secure new password for your partner account.'}
                    </p>
                </div>

                {#if form?.error}
                    <div class="error-banner animate-bounce">
                        <span class="err-icon">⚠️</span>
                        <p class="err-msg">{form.error}</p>
                    </div>
                {/if}

                {#if password && password.length < 8}
                    <div class="error-banner animate-fade-in" style="background: rgba(217, 119, 6, 0.05); border-color: rgba(217, 119, 6, 0.15); margin-bottom: 1rem;">
                        <span class="err-icon" style="color: #d97706;">⚠️</span>
                        <p class="err-msg" style="color: #b45309;">{i18n.locale === 'ar' ? 'يجب أن تتكون كلمة المرور من 8 أحرف على الأقل.' : 'Password must be at least 8 characters long.'}</p>
                    </div>
                {:else}
                    {#if password && confirmPassword && password !== confirmPassword}
                        <div class="error-banner animate-fade-in" style="background: rgba(239, 68, 68, 0.05); border-color: rgba(239, 68, 68, 0.15); margin-bottom: 1rem;">
                            <span class="err-icon">⚠️</span>
                            <p class="err-msg">{i18n.locale === 'ar' ? 'كلمتا السر غير متطابقتين.' : 'Passwords do not match.'}</p>
                        </div>
                    {/if}
                {/if}

                <form method="POST" use:enhance={handleSubmit} class="glassmorphic-card">
                    <input type="hidden" name="token" value={token} />

                    <div class="input-group">
                        <label for="password">{i18n.locale === 'ar' ? 'كلمة المرور الجديدة' : 'New Security Password'}</label>
                        <div class="wrapper">
                            <Lock class="icon" size={16} />
                            <input 
                                type="password" 
                                id="password" 
                                name="password" 
                                bind:value={password}
                                placeholder="••••••••" 
                                required 
                                autocomplete="new-password"
                            />
                            <div class="focus-glow"></div>
                        </div>
                    </div>

                    <div class="input-group">
                        <label for="confirmPassword">{i18n.locale === 'ar' ? 'تأكيد كلمة المرور' : 'Confirm New Password'}</label>
                        <div class="wrapper">
                            <Lock class="icon" size={16} />
                            <input 
                                type="password" 
                                id="confirmPassword" 
                                name="confirmPassword" 
                                bind:value={confirmPassword}
                                placeholder="••••••••" 
                                required 
                                autocomplete="new-password"
                            />
                            <div class="focus-glow"></div>
                        </div>
                    </div>

                    <button type="submit" class="submit-btn" disabled={password.length < 8 || password !== confirmPassword}>
                        {i18n.locale === 'ar' ? 'تأمين وحفظ كلمة المرور' : 'Secure & Save Password'} <ArrowRight size={16} />
                    </button>
                </form>

                <div class="form-navigation-footer">
                    <p><a href="/login" class="register-redirect-link">➔ {i18n.locale === 'ar' ? 'الرجوع لصفحة الدخول' : 'Back to Partner Login'}</a></p>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    /* premium split layout styling */
    .login-layout {
        display: grid;
        grid-template-columns: 42% 58%;
        min-height: calc(100vh - 120px);
        width: 100%;
        background-color: var(--color-bg-warm);
        position: relative;
        z-index: 10;
        overflow: hidden;
    }

    /* Left Hero Panel */
    .brand-panel {
        background: radial-gradient(circle at 100% 50%, rgba(91, 33, 182, 0.04) 0%, rgba(250, 248, 245, 0.95) 80%);
        border-inline-end: 1px solid rgba(0, 0, 0, 0.05);
        padding: 4rem 3.5rem;
        display: flex;
        align-items: center;
        position: relative;
    }

    .brand-panel::before {
        content: '';
        position: absolute;
        inset: 0;
        background-image: radial-gradient(rgba(217, 119, 6, 0.015) 1px, transparent 1px);
        background-size: 32px 32px;
        pointer-events: none;
    }

    .glow-orb {
        position: absolute;
        border-radius: 50%;
        filter: blur(120px);
        opacity: 0.12;
        pointer-events: none;
    }

    .gold-orb {
        width: 300px;
        height: 300px;
        background: var(--color-secondary);
        top: 15%;
        inset-inline-start: -10%;
    }

    .teal-orb {
        width: 350px;
        height: 350px;
        background: var(--color-primary);
        bottom: 15%;
        inset-inline-end: -10%;
    }

    .panel-content {
        position: relative;
        z-index: 10;
        display: flex;
        flex-direction: column;
        gap: 3.5rem;
        width: 100%;
    }

    .brand-header {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .gold-badge {
        align-self: flex-start;
        font-size: 0.7rem;
        font-weight: 700;
        letter-spacing: 2px;
        color: var(--color-secondary);
        background: rgba(217, 119, 6, 0.1);
        border: 1px solid rgba(217, 119, 6, 0.25);
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        text-transform: uppercase;
    }

    .brand-header h2 {
        font-size: 2.2rem;
        font-weight: 855;
        letter-spacing: -0.5px;
        color: var(--color-text-dark);
        margin: 0;
        line-height: 1.2;
    }

    .tagline {
        color: #475569;
        font-size: 1rem;
        margin: 0;
        line-height: 1.5;
    }

    .features-list {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .feature-item {
        display: flex;
        gap: 1.25rem;
        align-items: flex-start;
    }

    .feature-icon {
        width: 42px;
        height: 42px;
        border-radius: 10px;
        background: rgba(91, 33, 182, 0.08);
        border: 1px solid rgba(91, 33, 182, 0.2);
        color: var(--color-primary);
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        box-shadow: 0 4px 12px rgba(91, 33, 182, 0.05);
    }

    .feature-text h3 {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--color-text-dark);
        margin: 0 0 0.25rem 0;
    }

    .feature-text p {
        font-size: 0.85rem;
        color: #475569;
        margin: 0;
        line-height: 1.5;
    }

    .panel-footer {
        font-size: 0.8rem;
        color: #64748b;
        border-top: 1px solid rgba(0, 0, 0, 0.05);
        padding-top: 1.5rem;
    }

    /* Right Form panel */
    .form-panel {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 4rem 3rem;
        background: var(--color-bg-warm);
    }

    .form-wrapper {
        width: 100%;
        max-width: 460px;
        z-index: 10;
    }

    .login-header h1 {
        font-size: 2.2rem;
        font-weight: 800;
        color: var(--color-text-dark);
        margin: 0 0 0.5rem 0;
        letter-spacing: -0.5px;
    }

    .login-header .subtitle {
        color: #475569;
        font-size: 0.95rem;
        margin: 0 0 2.5rem 0;
        line-height: 1.5;
    }

    /* Glassmorphic Form Card block */
    .glassmorphic-card {
        background: var(--color-surface);
        border: 1px solid rgba(0, 0, 0, 0.06);
        border-radius: 16px;
        padding: 2.5rem;
        box-shadow: 0 20px 40px rgba(91, 33, 182, 0.04),
                    inset 0 1px 0 rgba(255, 255, 255, 0.5);
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .success-card {
        text-align: center;
        align-items: center;
        gap: 1rem;
    }

    .success-icon-container {
        width: 72px;
        height: 72px;
        background: rgba(22, 163, 74, 0.08);
        border: 1px solid rgba(22, 163, 74, 0.2);
        color: #16a34a;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 0.5rem;
    }

    .success-card h1 {
        font-size: 1.6rem;
        font-weight: 800;
        margin: 0;
        color: var(--color-text-dark);
    }

    .success-description {
        font-size: 0.9rem;
        color: #475569;
        line-height: 1.6;
        margin: 0 0 1rem 0;
    }

    .back-login-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem 1.5rem;
        background: var(--color-primary);
        color: white;
        border-radius: 8px;
        font-size: 0.9rem;
        font-weight: 700;
        text-decoration: none;
        box-shadow: 0 4px 12px rgba(91, 33, 182, 0.15);
        transition: all 0.2s ease;
    }

    .back-login-btn:hover {
        background: var(--color-primary-hover);
        transform: translateY(-1px);
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
    }

    .input-group label {
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--color-text-dark);
        letter-spacing: 0.5px;
    }

    .wrapper {
        position: relative;
        display: flex;
        align-items: center;
    }

    .wrapper :global(.icon) {
        position: absolute;
        inset-inline-start: 0.95rem;
        color: #64748b;
        pointer-events: none;
        z-index: 5;
    }

    .wrapper input {
        width: 100%;
        padding-top: 0.75rem;
        padding-bottom: 0.75rem;
        padding-inline-start: 2.65rem;
        padding-inline-end: 1rem;
        background: var(--color-surface);
        border: 1px solid #cbd5e1;
        border-radius: 8px;
        color: var(--color-text-dark);
        font-size: 0.9rem;
        font-family: inherit;
        outline: none;
        transition: all 0.25s ease;
    }

    .wrapper input:focus {
        border-color: var(--color-primary);
        background: var(--color-surface);
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.1);
    }

    .wrapper input:focus + .focus-glow {
        opacity: 1;
    }

    .focus-glow {
        position: absolute;
        top: 0;
        inset-inline-start: 0;
        width: 100%;
        height: 100%;
        border-radius: 8px;
        border: 1px solid var(--color-primary);
        box-shadow: 0 0 15px rgba(91, 33, 182, 0.08);
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.25s ease;
        z-index: 2;
    }

    .submit-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.85rem;
        border-radius: 8px;
        font-size: 0.95rem;
        font-weight: 700;
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
        margin-top: 0.5rem;
        background: var(--color-primary);
        color: #ffffff;
        box-shadow: 0 4px 12px rgba(91, 33, 182, 0.15);
    }

    .submit-btn:hover:not(:disabled) {
        background: var(--color-primary-hover);
        transform: translateY(-1px);
        box-shadow: 0 6px 16px rgba(91, 33, 182, 0.25);
    }

    .submit-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .form-navigation-footer {
        text-align: center;
        margin-top: 1.5rem;
        font-size: 0.85rem;
        color: #475569;
    }

    .register-redirect-link {
        color: var(--color-primary);
        font-weight: 600;
        text-decoration: none;
        transition: color 0.2s ease;
    }

    .register-redirect-link:hover {
        color: var(--color-primary-hover);
        text-decoration: underline;
    }

    /* Error Banner alert styling */
    .error-banner {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        background: rgba(239, 68, 68, 0.05);
        border: 1px solid rgba(239, 68, 68, 0.15);
        padding: 0.75rem 1rem;
        border-radius: 8px;
        margin-bottom: 1.5rem;
    }

    .err-msg {
        font-size: 0.85rem;
        color: #b91c1c;
        margin: 0;
        font-weight: 500;
    }


    /* Fluid Responsive adjustments */
    @media (max-width: 1024px) {
        .login-layout {
            grid-template-columns: 1fr;
        }

        .brand-panel {
            display: none;
        }

        .form-panel {
            padding: 3rem 1.5rem;
        }

        .glassmorphic-card {
            padding: 1.5rem;
        }
    }
</style>
