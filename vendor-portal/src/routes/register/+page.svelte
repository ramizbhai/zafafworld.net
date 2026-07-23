<script lang="ts">
    import { enhance } from '$app/forms';
    import { Building2, Mail, Phone, Lock, ArrowRight, Sparkles, ShieldCheck, Heart } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { uiStore } from '$lib/stores/ui.svelte';

    interface Props {
        data: any;
        form: {
            success?: boolean;
            error?: string;
            message?: string;
            slug?: string;
            values?: any;
        } | null;
    }

    let { data, form }: Props = $props();

    let loading = $state(false);

    let official_name = $state('');
    let email = $state('');
    let password = $state('');
    let phone = $state('');

    const i18n = getI18n();

    $effect(() => {
        if (form?.values) {
            official_name = form.values.official_name || '';
            email = form.values.email || '';
            phone = form.values.phone || '';
        }
    });
</script>

<svelte:head>
    <title>{i18n.locale === 'ar' ? 'تسجيل شريك جديد' : 'Vendor Registration'} | ZafafWorld</title>
</svelte:head>

<div class="login-layout" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <!-- Left Hero Brand Panel -->
    <div class="brand-panel">
        <div class="glow-orb gold-orb"></div>
        <div class="glow-orb teal-orb"></div>
        
        <div class="panel-content">
            <div class="brand-header">
                <span class="gold-badge">{i18n.locale === 'ar' ? 'انضم إلينا' : 'JOIN ZAFAF'}</span>
                <h2>{i18n.t.auth.growBusiness}</h2>
                <p class="tagline">{i18n.t.auth.growBusinessDesc}</p>
            </div>
            
            <div class="features-list">
                <div class="feature-item">
                    <div class="feature-icon">
                        <Sparkles size={20} />
                    </div>
                    <div class="feature-text">
                        <h3>{i18n.t.auth.directBookings}</h3>
                        <p>{i18n.t.auth.directBookingsDesc}</p>
                    </div>
                </div>
                
                <div class="feature-item">
                    <div class="feature-icon">
                        <ShieldCheck size={20} />
                    </div>
                    <div class="feature-text">
                        <h3>{i18n.t.auth.secureOnboarding}</h3>
                        <p>{i18n.t.auth.secureOnboardingDesc}</p>
                    </div>
                </div>
                
                <div class="feature-item">
                    <div class="feature-icon">
                        <Heart size={20} />
                    </div>
                    <div class="feature-text">
                        <h3>{i18n.locale === 'ar' ? 'الانتشار والنمو' : 'Marketplace Reach'}</h3>
                        <p>{i18n.locale === 'ar' ? 'انشر تفاصيل خدماتك وقاعاتك للوصول لشريحة أوسع من العملاء.' : 'Publish your venues and services to reach a wider audience of couples.'}</p>
                    </div>
                </div>
            </div>

            <div class="panel-footer">
                <p>{i18n.locale === 'ar' ? 'انضم إلى شبكة مزودي خدمات الزفاف الرائدة في المنطقة.' : 'Join the leading network of premium wedding service providers in the region.'}</p>
            </div>
        </div>
    </div>

    <!-- Right Register Form Panel -->
    <div class="form-panel">
        <div class="form-wrapper">
            {#if form?.success}
                <div class="success-card glassmorphic-card">
                    <div class="success-icon-wrapper animate-bounce">
                        <ShieldCheck class="success-icon" size={48} />
                    </div>
                    <h2>{i18n.t.auth.successTitle}</h2>
                    <p class="success-description">
                        {@html i18n.interpolate(i18n.t.auth.successDesc, { name: `<strong>${official_name || 'your business'}</strong>` })}
                    </p>
                    
                    <div class="next-steps">
                        <h3>{i18n.t.auth.nextSteps}</h3>
                        <ul>
                            <li>{i18n.t.auth.nextStep1}</li>
                            <li>{i18n.t.auth.nextStep2}</li>
                            <li>{i18n.t.auth.nextStep3}</li>
                        </ul>
                    </div>

                    <a href="/login" class="submit-btn">
                        {i18n.t.auth.goLoginBtn}
                        <ArrowRight size={16} />
                    </a>
                </div>
            {:else}
                <div class="login-header">
                    <h1>{i18n.t.auth.corporateOnboarding}</h1>
                    <p class="subtitle">{i18n.t.auth.onboardingDesc}</p>
                </div>

                {#if form?.error}
                    <div class="error-banner animate-bounce">
                        <span class="err-icon">⚠️</span>
                        <p class="err-msg">{form.error}</p>
                    </div>
                {/if}

                <form method="POST" use:enhance={() => {
                    uiStore.setLoading(true);
                    return async ({ update }) => {
                        await update();
                        uiStore.setLoading(false);
                    };
                }} class="glassmorphic-card">
                    <div class="input-group">
                        <label for="official_name">{i18n.t.auth.companyName}</label>
                        <div class="wrapper">
                            <Building2 class="icon" size={16} />
                            <input 
                                id="official_name"
                                name="official_name" 
                                type="text" 
                                placeholder={i18n.t.auth.companyNamePl} 
                                bind:value={official_name}
                                required 
                            />
                            <div class="focus-glow"></div>
                        </div>
                    </div>

                    <div class="input-group">
                        <label for="email">{i18n.t.auth.corporateEmailField}</label>
                        <div class="wrapper">
                            <Mail class="icon" size={16} />
                            <input 
                                id="email"
                                name="email" 
                                type="email" 
                                placeholder={i18n.t.auth.corporateEmailPlaceholder || 'vendor@yourdomain.com'} 
                                bind:value={email}
                                oninput={() => { email = email.toLowerCase(); }}
                                required 
                                autocomplete="email"
                            />
                            <div class="focus-glow"></div>
                        </div>
                    </div>

                    <div class="input-group">
                        <label for="phone">{i18n.t.auth.phoneField}</label>
                        <div class="wrapper">
                            <Phone class="icon" size={16} />
                            <input 
                                id="phone"
                                name="phone" 
                                type="tel" 
                                placeholder={i18n.t.auth.phonePlaceholder || '+966 50 123 4567'} 
                                bind:value={phone}
                                required 
                                autocomplete="tel"
                            />
                            <div class="focus-glow"></div>
                        </div>
                    </div>

                    <div class="input-group">
                        <label for="password">{i18n.t.auth.securityPassword}</label>
                        <div class="wrapper">
                            <Lock class="icon" size={16} />
                            <input 
                                id="password"
                                name="password" 
                                type="password" 
                                placeholder="••••••••" 
                                bind:value={password}
                                required 
                                minlength={8}
                                autocomplete="new-password"
                            />
                            <div class="focus-glow"></div>
                        </div>
                        <div class="hint-container">
                            <span class="hint-text">{i18n.t.auth.passwordHint}</span>
                        </div>
                    </div>

                    <button type="submit" class="submit-btn">
                        {i18n.t.auth.registerBtn} <ArrowRight size={16} />
                    </button>
                </form>

                <div class="form-navigation-footer">
                    <p>
                        {i18n.t.auth.haveAccount} 
                        <a href="/login" class="login-redirect-link">{i18n.t.auth.goToLogin}</a>
                    </p>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    /* Premium Page Split Layout styling */
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

    .hint-container {
        margin-top: 0.25rem;
    }

    .hint-text {
        font-size: 0.75rem;
        color: #64748b;
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
        text-decoration: none;
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

    .login-redirect-link {
        color: var(--color-primary);
        font-weight: 600;
        text-decoration: none;
        transition: color 0.2s ease;
    }

    .login-redirect-link:hover {
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


    /* Success State Styles */
    .success-card {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .success-icon-wrapper {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 4rem;
        height: 4rem;
        border-radius: 50%;
        background-color: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.25);
        color: #10b981;
        margin-bottom: 1.5rem;
    }

    .success-card h2 {
        font-size: 1.5rem;
        font-weight: 800;
        color: var(--color-text-dark);
        margin: 0 0 0.75rem 0;
    }

    .success-description {
        color: #475569;
        font-size: 0.95rem;
        line-height: 1.6;
        margin: 0 0 2rem 0;
    }

    .next-steps {
        background-color: rgba(0, 0, 0, 0.02);
        border: 1px solid rgba(0, 0, 0, 0.06);
        border-radius: 12px;
        padding: 1.25rem 1.5rem;
        width: 100%;
        text-align: start;
        margin-bottom: 2rem;
    }

    .next-steps h3 {
        font-size: 0.85rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--color-primary);
        margin-top: 0;
        margin-bottom: 0.75rem;
    }

    .next-steps ul {
        margin: 0;
        padding-inline-start: 1.25rem;
        color: #475569;
        font-size: 0.85rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .next-steps li {
        line-height: 1.4;
    }

    @keyframes bounce {
        0%, 100% {
            transform: translateY(0);
        }
        50% {
            transform: translateY(-8px);
        }
    }

    .animate-bounce {
        animation: bounce 2s infinite;
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
